use crate::models::Contestant;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum ContestantServiceError {
    #[error("Contestant not found")]
    NotFound,
    #[error("unexpected error, {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[async_trait]
pub trait ContestantService {
    async fn get_contestants(&self) -> Result<Vec<Contestant>, ContestantServiceError>;
    async fn get_contestant(&self, discord_id: &str) -> Result<Contestant, ContestantServiceError> {
        self.get_contestants()
            .await?
            .into_iter()
            .find(|c| c.discord_id == discord_id)
            .ok_or(ContestantServiceError::NotFound)
    }
}

pub struct FakeContestantService;

#[async_trait]
impl ContestantService for FakeContestantService {
    async fn get_contestants(&self) -> Result<Vec<Contestant>, ContestantServiceError> {
        Ok(vec![])
    }
}
