use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::StatusCode;
use serenity::http::Http;
use serenity::model::prelude::Embed;
use serenity::model::webhook::Webhook;
use serenity::utils::Colour;

type RedeployResult = Result<String, RedeployError>;

#[async_trait]
pub trait RedeployService {
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult;
}

#[derive(Debug)]
pub struct RedeployTarget {
    pub team_id: String,
    pub problem_id: String,
}

#[async_trait]
pub trait RedeployNotifier {
    async fn notify(&self, target: &RedeployTarget, result: &RedeployResult);
}

#[derive(Debug, thiserror::Error)]
pub enum RedeployError {
    #[error("invalid parameters")]
    InvalidParameters,
    #[error("another job is in queue")]
    AnotherJobInQueue(String),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
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
}

impl RState {
    pub fn new(config: RStateConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .user_agent("ICTSC Discord Bot")
            .build()?;

        Ok(Self { config, client })
    }
}

#[derive(Debug, serde::Serialize)]
struct DefaultRedeployServiceRedeployRequest<'a> {
    team_id: &'a str,
    prob_id: &'a str,
}

#[async_trait]
impl RedeployService for RState {
    #[tracing::instrument(skip_all, fields(target = ?target))]
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult {
        tracing::info!("redeploy request received");

        let response = self
            .client
            .post(format!("{}/admin/postJob", self.config.baseurl))
            .form(&DefaultRedeployServiceRedeployRequest {
                team_id: &target.team_id,
                prob_id: &target.problem_id,
            })
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let url = String::from_utf8(response.bytes().await?.to_vec())
                    .map_err(|err| RedeployError::Unexpected(Box::new(err)))?;
                Ok(url)
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
}

pub struct FakeRedeployService;

#[async_trait]
impl RedeployService for FakeRedeployService {
    #[tracing::instrument(skip_all, fields(target = ?_target))]
    async fn redeploy(&self, _target: &RedeployTarget) -> RedeployResult {
        tracing::info!("redeploy request received");
        Ok(String::from("https://example.com"))
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
    async fn notify(&self, target: &RedeployTarget, result: &RedeployResult) {
        if let Err(err) = self._notify(target, result).await {
            tracing::error!("failed to notify: {:?}", err)
        }
    }
}

impl DiscordRedeployNotifier {
    async fn _notify(&self, target: &RedeployTarget, result: &RedeployResult) -> Result<()> {
        let embed = match result {
            Ok(url) => Embed::fake(|e| {
                e.title("AAA")
                    .colour(Colour::from_rgb(40, 167, 65))
                    .field("チームID", &target.team_id, true)
                    .field("問題コード", &target.problem_id, true)
                    .field("再展開進捗URL", url, true)
            }),
            Err(err) => Embed::fake(|e| {
                e.title("AAA")
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
