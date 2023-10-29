use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, StatusCode};

type RedeployResult = Result<(), RedeployError>;

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
    #[error("another redeploy job is in queue")]
    AnotherRedeployJobInQueue,
    #[error("out of competition time")]
    OutOfCompetitionTime,
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("unexpected error occured: {0}")]
    UnexpectedError(String),
}

// TODO: 後でいい感じの名前に変える
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
                // let data = response.bytes().await?.to_vec();
                // Ok(format!(
                //     "{}{}",
                //     self.baseurl,
                //     String::from_utf8(data).unwrap()
                // ))
                Ok(())
            }
            StatusCode::BAD_REQUEST => Err(RedeployError::AnotherRedeployJobInQueue),
            StatusCode::NOT_FOUND => Err(RedeployError::OutOfCompetitionTime),
            _ => Err(RedeployError::UnexpectedError(format!(
                "unexpected status code {} returned from upstream server",
                response.status()
            ))),
        }
    }
}

pub struct FakeRedeployService;

#[async_trait]
impl RedeployService for FakeRedeployService {
    #[tracing::instrument(skip_all, fields(target = ?_target))]
    async fn redeploy(&self, _target: RedeployTarget) -> RedeployResult {
        tracing::info!("redeploy request received");
        Ok(())
    }
}
