use async_trait::async_trait;
use serde_derive::Deserialize;
use crate::services::redeploy::regalia::Regalia;

#[derive(Debug, Clone, Deserialize)]
pub struct Contestant {
    pub name: String,
    pub display_name: String,
    pub team: Team,
    pub profile: Profile,
    pub discord_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub self_introduction: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub code: usize,
    pub name: String,
    pub organization: String,
    pub member_limit: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ContestantError {
    #[error("Contestant not found")]
    NotFound,
    #[error("unexpected error, {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[async_trait]
pub trait ContestantService {
    async fn get_contestants(&self) -> Result<Vec<Contestant>, ContestantError>;
    async fn get_contestant(&self, discord_id: String) -> Result<Contestant, ContestantError> {
        self.get_contestants()
            .await?
            .into_iter()
            .find(|c| c.discord_id == discord_id)
            .ok_or(ContestantError::NotFound)
    }
}

