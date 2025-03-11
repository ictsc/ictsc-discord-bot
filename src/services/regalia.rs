use crate::models::{Contestant, Problem, Team};
use crate::services::contestant::{ContestantError, ContestantService};
use crate::services::problem::{ProblemError, ProblemService};
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

    pub async fn list_problems(&self) -> anyhow::Result<Vec<Problem>, ProblemError> {
        let response = self
            .client
            .post(format!(
                "{}ProblemService/ListProblems",
                self.config.baseurl
            ))
            .json(&RegaliaPostListProblemsRequest {})
            .send()
            .await
            .map_err(|e| ProblemError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                let problems = response
                    .json::<RegaliaPostListProblemsResponse>()
                    .await
                    .map_err(|e| ProblemError::Unexpected(Box::new(e)))?
                    .problems
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Ok(problems)
            },
            _ => Err(ProblemError::Unexpected(Box::new(
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

#[async_trait]
impl ProblemService for Regalia {
    async fn get_problems(&self) -> Result<Vec<Problem>, ProblemError> {
        self.list_problems().await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaContestant {
    name: String,
    display_name: String,
    team: RegaliaTeam,
    #[allow(dead_code)]
    profile: RegaliaProfile,
    discord_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaProfile {
    #[serde(default)]
    #[allow(dead_code)]
    self_introduction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaTeam {
    code: String,
    name: String,
    #[allow(dead_code)]
    organization: String,
    #[allow(dead_code)]
    member_limit: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaProblem {
    code: String,
    title: String,
    #[allow(dead_code)]
    max_score: u32,
    #[allow(dead_code)]
    redeploy_rule: RegaliaRedeployRule,
    #[allow(dead_code)]
    body: RegaliaProblemBody,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaRedeployRule {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    typ: RegaliaRedeployRuleType,
    #[serde(default)]
    #[allow(dead_code)]
    penalty_threshold: u32,
    #[serde(default)]
    #[allow(dead_code)]
    penalty_percentage: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RegaliaRedeployRuleType {
    RedeployRuleTypeUnspecified,
    RedeployRuleTypeUnredeployable,
    RedeployRuleTypePercentagePenalty,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaProblemBody {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    typ: RegaliaProblemType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RegaliaProblemType {
    ProblemTypeUnspecified,
    ProblemTypeDescriptive,
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

impl From<RegaliaProblem> for Problem {
    fn from(value: RegaliaProblem) -> Self {
        Self {
            code: value.code,
            name: value.title,
        }
    }
}

#[derive(Debug, Serialize)]
struct RegaliaPostListAllContestantsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListAllContestantsResponse {
    contestants: Vec<RegaliaContestant>,
}

#[derive(Debug, Serialize)]
struct RegaliaPostListAllTeamsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListAllTeamsResponse {
    teams: Vec<RegaliaTeam>,
}

#[derive(Debug, Serialize)]
struct RegaliaPostListProblemsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListProblemsResponse {
    problems: Vec<RegaliaProblem>,
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
