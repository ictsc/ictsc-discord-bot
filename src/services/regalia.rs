use crate::models::{Contestant, Team};
use crate::services::contestant::{ContestantError, ContestantService};
use crate::services::redeploy::{
    RedeployJob, RedeployResult, RedeployService, RedeployStatusList, RedeployTarget,
};
use crate::services::team::{TeamError, TeamService};
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder};
use serde_derive::{Deserialize, Serialize};

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
        let header_map = HeaderMap::from_iter([(
            "Authorization".parse()?,
            format!("Bearer {}", config.token).parse()?,
        )]);
        let client = ClientBuilder::new()
            .user_agent("ICTSC Discord Bot")
            .default_headers(header_map)
            .gzip(true)
            .build()?;

        Ok(Self { config, client })
    }

    pub async fn list_contestants(&self) -> anyhow::Result<Vec<Contestant>, ContestantError> {
        let response = self
            .client
            .post(format!(
                "{}ContestantService/ListContestants",
                self.config.baseurl
            ))
            .json(&RegaliaPostListAllContestantsRequest {})
            .send()
            .await
            .map_err(|e| ContestantError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                let contestants = response
                    .json::<RegaliaPostListAllContestantsResponse>()
                    .await
                    .map_err(|e| ContestantError::Unexpected(Box::new(e)))?
                    .contestants
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>();
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

    pub async fn list_teams(&self) -> anyhow::Result<Vec<Team>, TeamError> {
        let response = self
            .client
            .post(format!("{}TeamService/ListTeams", self.config.baseurl))
            .json(&RegaliaPostListAllTeamsRequest {})
            .send()
            .await
            .map_err(|e| TeamError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                let teams = response
                    .json::<RegaliaPostListAllTeamsResponse>()
                    .await
                    .map_err(|e| TeamError::Unexpected(Box::new(e)))?
                    .teams
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>();
                Ok(teams)
            },
            _ => Err(TeamError::Unexpected(Box::new(
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
impl TeamService for Regalia {
    async fn get_teams(&self) -> Result<Vec<Team>, TeamError> {
        self.list_teams().await
    }
}

#[derive(Debug, Serialize)]
struct RegaliaPostListAllContestantsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListAllContestantsResponse {
    contestants: Vec<RegaliaContestant>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaContestant {
    name: String,
    display_name: String,
    team: RegaliaTeam,
    profile: RegaliaProfile,
    discord_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaProfile {
    #[serde(default)]
    self_introduction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaTeam {
    code: String,
    name: String,
    organization: String,
    member_limit: u32,
}

impl From<RegaliaContestant> for Contestant {
    fn from(value: RegaliaContestant) -> Self {
        Self {
            name: value.name,
            display_name: value.display_name,
            team_id: value.team.code,
            discord_id: value.discord_id,
        }
    }
}

impl From<RegaliaTeam> for Team {
    fn from(value: RegaliaTeam) -> Self {
        Self {
            id: value.code,
            role_name: value.name,
            invitation_code: "".to_string(),
            user_group_id: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RegaliaPostListAllTeamsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListAllTeamsResponse {
    teams: Vec<RegaliaTeam>,
}

#[async_trait]
impl RedeployService for Regalia {
    async fn redeploy(&self, _target: &RedeployTarget) -> RedeployResult<RedeployJob> {
        todo!()
    }

    async fn get_status(&self, _team_id: &str) -> RedeployResult<RedeployStatusList> {
        todo!()
    }
}
