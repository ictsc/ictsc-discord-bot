use crate::services::contestant::{Contestant, ContestantError, ContestantService};
use crate::services::redeploy::{
    RedeployJob, RedeployResult, RedeployService, RedeployStatusList, RedeployTarget,
};
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder};
use serde_derive::Serialize;

pub struct Regalia {
    config: RegaliaConfig,
    client: Client,
}

pub struct RegaliaConfig {
    pub baseurl: String,
    pub token: String,
}

impl Regalia {
    pub fn new(config: RegaliaConfig) -> anyhow::Result<Self> {
        let header_map = HeaderMap::from_iter([
            (
                "Content-Type".parse().unwrap(),
                "application/json;charset=utf-8".parse().unwrap(),
            ),
            (
                "Authorization".parse().unwrap(),
                format!("Bearer {}", config.token).parse().unwrap(),
            ),
        ]);
        let client = ClientBuilder::new()
            .user_agent("ICTSC Discord Bot")
            .default_headers(header_map)
            .build()?;

        Ok(Self { config, client })
    }

    pub async fn list_contestants(&self) -> anyhow::Result<Vec<Contestant>, ContestantError> {
        let response = self
            .client
            .post(format!("{}Contestant/ListContestants", self.config.baseurl))
            .form(&RegaliaPostJobRequest { team_code: 1 })
            .send()
            .await
            .map_err(|e| ContestantError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                let contestants = response
                    .json::<Vec<Contestant>>()
                    .await
                    .map_err(|e| ContestantError::Unexpected(Box::new(e)))?;
                if contestants.is_empty() {
                    Err(ContestantError::NotFound)
                } else {
                    Ok(contestants)
                }
            },
            _ => Err(ContestantError::Unexpected(Box::new(
                response.error_for_status().unwrap_err(),
            ))),
        }
    }
}

#[async_trait]
impl ContestantService for Regalia {
    async fn get_contestants(&self) -> Result<Vec<Contestant>, ContestantError> {
        self.list_contestants().await
    }
}

#[async_trait]
impl RedeployService for Regalia {
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult<RedeployJob> {
        todo!()
    }

    async fn get_status(&self, team_id: &str) -> RedeployResult<RedeployStatusList> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
struct RegaliaPostJobRequest {
    team_code: i64,
}
