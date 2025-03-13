use crate::models::{Contestant, Problem, Team};
use crate::services::contestant::{ContestantService, ContestantServiceError};
use crate::services::problem::{ProblemError, ProblemService};
use crate::services::redeploy::{
    RedeployError, RedeployJob, RedeployResult, RedeployService, RedeployStatus,
    RedeployStatusList, RedeployTarget,
};
use crate::services::team::{TeamError, TeamService};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
}

#[async_trait]
impl ContestantService for Regalia {
    #[tracing::instrument(skip_all)]
    async fn get_contestants(&self) -> anyhow::Result<Vec<Contestant>, ContestantServiceError> {
        let response = self
            .client
            .post(format!(
                "{}admin.v1.ContestantService/ListContestants",
                self.config.baseurl
            ))
            .json(&RegaliaPostListAllContestantsRequest {})
            .send()
            .await
            .map_err(|e| ContestantServiceError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                let contestants = response
                    .json::<RegaliaPostListAllContestantsResponse>()
                    .await
                    .map_err(|e| ContestantServiceError::Unexpected(Box::new(e)))?
                    .contestants
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>();
                if contestants.is_empty() {
                    Err(ContestantServiceError::NotFound)
                } else {
                    Ok(contestants)
                }
            },
            _ => Err(ContestantServiceError::Unexpected(Box::new(
                response.error_for_status().unwrap_err(),
            ))),
        }
    }
}

#[async_trait]
impl TeamService for Regalia {
    #[tracing::instrument(skip_all)]
    async fn get_teams(&self) -> anyhow::Result<Vec<Team>, TeamError> {
        let response = self
            .client
            .post(format!(
                "{}admin.v1.TeamService/ListTeams",
                self.config.baseurl
            ))
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
impl ProblemService for Regalia {
    #[tracing::instrument(skip_all)]
    async fn get_problems(&self) -> anyhow::Result<Vec<Problem>, ProblemError> {
        let response = self
            .client
            .post(format!(
                "{}admin.v1.ProblemService/ListProblems",
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
impl RedeployService for Regalia {
    #[tracing::instrument(skip_all, fields(target = ?target))]
    async fn redeploy(&self, target: &RedeployTarget) -> RedeployResult<RedeployJob> {
        let team_code = target.team_id.clone();
        let problem_code = target.problem_id.clone();
        let response = self
            .client
            .post(format!(
                "{}admin.v1.DeploymentService/Deploy",
                self.config.baseurl
            ))
            .json(&RegaliaPostDeployRequest {
                team_code,
                problem_code,
            })
            .send()
            .await
            .map_err(|e| RedeployError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(response
                .json::<RegaliaPostDeployResponse>()
                .await
                .map_err(|e| RedeployError::Unexpected(Box::new(e)))?
                .deployment
                .into()),
            _ => Err(RedeployError::Unexpected(Box::new(
                response.error_for_status().unwrap_err(),
            ))),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_status(&self, team_id: &str) -> RedeployResult<RedeployStatusList> {
        let team_code = team_id.to_string();
        let response = self
            .client
            .post(format!(
                "{}admin.v1.DeploymentService/ListDeployments",
                self.config.baseurl
            ))
            .json(&RegaliaPostListDeploymentsRequest { team_code })
            .send()
            .await
            .map_err(|e| RedeployError::Unexpected(Box::new(e)))?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(response
                .json::<RegaliaPostListDeploymentsResponse>()
                .await
                .map_err(|e| RedeployError::Unexpected(Box::new(e)))?
                .deployments
                .into_iter()
                .map(Into::into)
                .collect()),
            _ => Err(RedeployError::Unexpected(Box::new(
                response.error_for_status().unwrap_err(),
            ))),
        }
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

#[derive(Debug, Serialize)]
struct RegaliaPostListAllContestantsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListAllContestantsResponse {
    #[serde(default)]
    contestants: Vec<RegaliaContestant>,
}

#[derive(Debug, Serialize)]
struct RegaliaPostListAllTeamsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListAllTeamsResponse {
    #[serde(default)]
    teams: Vec<RegaliaTeam>,
}

#[derive(Debug, Serialize)]
struct RegaliaPostListProblemsRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListProblemsResponse {
    #[serde(default)]
    problems: Vec<RegaliaProblem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListDeploymentsRequest {
    team_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostListDeploymentsResponse {
    #[serde(default)]
    deployments: Vec<RegaliaDeployment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostDeployRequest {
    team_code: String,
    problem_code: String,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaPostDeployResponse {
    deployment: RegaliaDeployment,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaDeployment {
    team_code: String,
    problem_code: String,
    #[allow(dead_code)]
    revision: i64,
    latest_event: RegaliaDeploymentEventType,
    #[serde(default)]
    events: Vec<RegaliaDeploymentEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RegaliaDeploymentEventType {
    DeploymentEventTypeUnspecified,
    DeploymentEventTypeQueued,
    DeploymentEventTypeCreating,
    DeploymentEventTypeFinished,
    DeploymentEventTypeError,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegaliaDeploymentEvent {
    occurred_at: DateTime<Utc>,
    #[serde(rename = "type")]
    typ: RegaliaDeploymentEventType,
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

impl From<RegaliaDeployment> for RedeployJob {
    fn from(value: RegaliaDeployment) -> Self {
        Self {
            id: "".to_string(),
            team_id: value.team_code,
            problem_code: value.problem_code,
        }
    }
}

impl From<RegaliaDeployment> for RedeployStatus {
    fn from(value: RegaliaDeployment) -> Self {
        use RegaliaDeploymentEventType::*;
        let team_id = value.team_code.clone();
        let problem_code = value.problem_code.clone();
        let is_redeploying = matches!(value.latest_event, DeploymentEventTypeCreating);
        let events = {
            let mut events = value.events.iter().collect::<Vec<_>>();
            events.sort_by(|a, b| a.occurred_at.cmp(&b.occurred_at));
            events.reverse();
            events
        };
        let last_redeploy_started_at = events
            .iter()
            .find_map(|e| matches!(e.typ, DeploymentEventTypeCreating).then(|| e.occurred_at));
        let last_redeploy_completed_at = events
            .iter()
            .find_map(|e| matches!(e.typ, DeploymentEventTypeFinished).then(|| e.occurred_at));
        Self {
            team_id,
            problem_code,
            is_redeploying,
            last_redeploy_started_at,
            last_redeploy_completed_at,
        }
    }
}
