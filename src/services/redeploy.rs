use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, StatusCode};

type RedeployResult = Result<String, RedeployError>;

#[async_trait]
pub trait RedeployService {
    async fn redeploy(&self, target: RedeployTarget) -> RedeployResult;
}

#[derive(Debug)]
pub struct RedeployTarget {
    pub team_id: String,
    pub problem_id: String,
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
struct DefaultRedeployServiceRedeployRequest {
    team_id: String,
    prob_id: String,
}

#[async_trait]
impl RedeployService for RState {
    #[tracing::instrument(skip_all, fields(target = ?target))]
    async fn redeploy(&self, target: RedeployTarget) -> RedeployResult {
        tracing::info!("redeploy request received");

        let response = self
            .client
            .post(format!("{}/admin/postJob", self.config.baseurl))
            .form(&DefaultRedeployServiceRedeployRequest {
                team_id: target.team_id,
                prob_id: target.problem_id,
            })
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let url = String::from_utf8(response.bytes().await?.to_vec())
                    .map_err(|err| RedeployError::Unexpected(Box::new(err)))?;
                Ok(url)
            }
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
    async fn redeploy(&self, _target: RedeployTarget) -> RedeployResult {
        tracing::info!("redeploy request received");
        Ok(String::from("https://example.com"))
    }
}
