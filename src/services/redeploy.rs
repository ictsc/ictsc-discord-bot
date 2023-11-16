use anyhow::Result;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serenity::http::Http;
use serenity::model::prelude::Embed;
use serenity::model::webhook::Webhook;
use serenity::utils::Colour;

use crate::models::Problem;

#[derive(Debug, Clone)]
pub struct RedeployJob {
    pub id: String,
    pub team_id: String,
    pub problem_code: String,
}

type RedeployStatusList = Vec<RedeployStatus>;

#[derive(Debug, Clone)]
pub struct RedeployStatus {
    pub team_id: String,
    pub problem_code: String,

    // 再展開中かを表すフラグ
    pub is_redeploying: bool,

    // 最後の再展開が開始された時刻
    pub last_redeploy_started_at: Option<DateTime<Utc>>,

    // 最後の再展開が完了した時刻
    pub last_redeploy_completed_at: Option<DateTime<Utc>>,
}

type RedeployResult<T> = Result<T, RedeployError>;

#[async_trait]
pub trait RedeployService {
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult<RedeployJob>;
    async fn get_status(&self, team_id: &str) -> RedeployResult<RedeployStatusList>;
}

#[derive(Debug)]
pub struct RedeployTarget {
    pub team_id: String,
    pub problem_id: String,
}

#[async_trait]
pub trait RedeployNotifier {
    async fn notify(&self, target: &RedeployTarget, result: &RedeployResult<RedeployJob>);
}

#[derive(Debug, thiserror::Error)]
pub enum RedeployError {
    #[error("invalid parameters")]
    InvalidParameters,
    #[error("another job is in queue")]
    AnotherJobInQueue(String),

    // serde_jsonでserialize/deserializeに失敗した時に出るエラー
    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),

    // reqwestでHTTP接続に失敗した時に出るエラー
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    // なんだかよくわからないエラー
    #[error("unexpected error occured: {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub struct RState {
    config: RStateConfig,
    client: Client,
}

pub struct RStateConfig {
    pub baseurl: String,
    pub username: String,
    pub password: String,
    pub problems: Vec<Problem>,
}

impl RState {
    pub fn new(config: RStateConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .user_agent("ICTSC Discord Bot")
            .build()?;

        Ok(Self { config, client })
    }
}

#[derive(Debug, Serialize)]
struct RStatePostJobRequest<'a> {
    team_id: &'a str,
    prob_id: &'a str,
}

#[derive(Debug, Deserialize)]
struct RStatePostJobResponse {
    id: String,
    team_id: String,
    prob_id: String,
}

#[derive(Debug, Deserialize)]
struct RStateGetRedeployStatusResponse {
    available: bool,
    created_time: Option<DateTime<Utc>>,
    completed_time: Option<DateTime<Utc>>,
}

#[async_trait]
impl RedeployService for RState {
    #[tracing::instrument(skip_all, fields(target = ?target))]
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult<RedeployJob> {
        tracing::info!("redeploy request received");

        let response = self
            .client
            .post(format!("{}/admin/postJob", self.config.baseurl))
            .form(&RStatePostJobRequest {
                team_id: &target.team_id,
                prob_id: &target.problem_id,
            })
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response: RStatePostJobResponse =
                    serde_json::from_slice(response.bytes().await?.as_ref())?;

                Ok(RedeployJob {
                    id: response.id,
                    team_id: response.team_id,
                    problem_code: response.prob_id,
                })
            },
            StatusCode::BAD_REQUEST => {
                let data = String::from_utf8(response.bytes().await?.to_vec())
                    .map_err(|err| RedeployError::Unexpected(Box::new(err)))?;

                if data == "BadRequest!" {
                    return Err(RedeployError::InvalidParameters);
                }

                Err(RedeployError::AnotherJobInQueue(data))
            },
            _ => Err(RedeployError::Unexpected(
                anyhow::anyhow!("unexpected status code: {}", response.status()).into(),
            )),
        }
    }

    #[tracing::instrument(skip_all, fields(team_id = ?team_id))]
    async fn get_status(&self, team_id: &str) -> RedeployResult<RedeployStatusList> {
        tracing::trace!("get_status request received");

        let mut statuses = Vec::new();
        for problem in &self.config.problems {
            let response = self
                .client
                .get(format!(
                    "{}/backend/{}/{}",
                    self.config.baseurl, team_id, problem.code
                ))
                .send()
                .await?;

            // /backend/statusは常に200を返すので、エラーハンドリングしない
            let response: RStateGetRedeployStatusResponse =
                serde_json::from_slice(response.bytes().await?.as_ref())?;

            statuses.push(RedeployStatus {
                team_id: team_id.to_string(),
                problem_code: problem.code.clone(),
                is_redeploying: !response.available,
                last_redeploy_started_at: response.created_time,
                last_redeploy_completed_at: response.completed_time,
            });
        }

        Ok(statuses)
    }
}

pub struct FakeRedeployService;

#[async_trait]
impl RedeployService for FakeRedeployService {
    #[tracing::instrument(skip_all, fields(target = ?target))]
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult<RedeployJob> {
        tracing::info!("redeploy request received");
        Ok(RedeployJob {
            id: String::from("00000000-0000-0000-0000-000000000000"),
            team_id: target.team_id.clone(),
            problem_code: target.problem_id.clone(),
        })
    }

    #[tracing::instrument(skip_all, fields(team_id = ?team_id))]
    async fn get_status(&self, team_id: &str) -> RedeployResult<RedeployStatusList> {
        tracing::trace!("get_status request received");

        let now = Utc::now();

        Ok(vec![
            RedeployStatus {
                team_id: team_id.to_string(),
                problem_code: String::from("ABC"),
                is_redeploying: false,
                last_redeploy_started_at: None,
                last_redeploy_completed_at: None,
            },
            RedeployStatus {
                team_id: team_id.to_string(),
                problem_code: String::from("DEF"),
                is_redeploying: true,
                last_redeploy_started_at: Some(now),
                last_redeploy_completed_at: None,
            },
            RedeployStatus {
                team_id: team_id.to_string(),
                problem_code: String::from("GHI"),
                is_redeploying: false,
                last_redeploy_started_at: Some(now),
                last_redeploy_completed_at: Some(now),
            },
        ])
    }
}

#[derive(Debug)]
pub struct DiscordRedeployNotifier {
    discord_client: Http,
    webhook: Webhook,
}

impl DiscordRedeployNotifier {
    pub async fn new(token: &str, webhook_url: &str) -> Result<Self> {
        let discord_client = Http::new(token);
        let webhook = Webhook::from_url(&discord_client, webhook_url).await?;
        Ok(Self {
            discord_client,
            webhook,
        })
    }
}

#[async_trait]
impl RedeployNotifier for DiscordRedeployNotifier {
    #[tracing::instrument(skip_all, fields(target = ?target, result = ?result))]
    async fn notify(&self, target: &RedeployTarget, result: &RedeployResult<RedeployJob>) {
        if let Err(err) = self._notify(target, result).await {
            tracing::error!("failed to notify: {:?}", err)
        }
    }
}

impl DiscordRedeployNotifier {
    async fn _notify(
        &self,
        target: &RedeployTarget,
        result: &RedeployResult<RedeployJob>,
    ) -> Result<()> {
        let embed = match result {
            Ok(job) => Embed::fake(|e| {
                e.title("再展開開始通知")
                    .colour(Colour::from_rgb(40, 167, 65))
                    .field("チームID", &target.team_id, true)
                    .field("問題コード", &target.problem_id, true)
                    .field("再展開Job ID", &job.id, true)
            }),
            Err(err) => Embed::fake(|e| {
                e.title("再展開失敗通知")
                    .colour(Colour::from_rgb(236, 76, 82))
                    .field("チームID", &target.team_id, true)
                    .field("問題コード", &target.problem_id, true)
                    .field("エラー", err, true)
            }),
        };

        let result = self
            .webhook
            .execute(&self.discord_client, false, |w| w.embeds(vec![embed]))
            .await?;

        if let Some(message) = result {
            tracing::debug!(message_id = ?message.id, "finished to notify redeploy event")
        }

        Ok(())
    }
}
