use async_trait::async_trait;
use crate::models::Team;

#[derive(Debug, thiserror::Error)]
pub enum TeamError {
    #[error("Team not found")]
    NotFound,
    #[error("unexpected error, {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[async_trait]
pub trait TeamService {
    async fn get_teams(&self) -> Result<Vec<Team>, TeamError>;
    async fn get_team(&self, team_id: &str) -> Result<Team, TeamError> {
        self.get_teams()
            .await?
            .into_iter()
            .find(|t| t.id == team_id)
            .ok_or(TeamError::NotFound)
    }
}

pub struct StaticTeamService {
    teams: Vec<Team>,
}

impl StaticTeamService {
    pub fn new(teams: Vec<Team>) -> Self {
        Self { teams }
    }
}

#[async_trait]
impl TeamService for StaticTeamService {
    async fn get_teams(&self) -> Result<Vec<Team>, TeamError> {
        Ok(self.teams.clone())
    }
}
