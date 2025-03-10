use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Contestant {
    pub name: String,
    pub display_name: String,
    pub team: Team,
    pub profile: Profile,
    pub discord_id: String,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub self_introduction: String,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub code: String,
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
    async fn get_contestant(&self, discord_id: &str) -> Result<Contestant, ContestantError> {
        self.get_contestants()
            .await?
            .into_iter()
            .find(|c| c.discord_id == discord_id)
            .ok_or(ContestantError::NotFound)
    }
}

pub struct FakeContestantService;

#[async_trait]
impl ContestantService for FakeContestantService {
    async fn get_contestants(&self) -> Result<Vec<Contestant>, ContestantError> {
        Ok(vec![
            Contestant {
                name: "Alice".to_string(),
                display_name: "Alice".to_string(),
                team: Team {
                    code: "1".to_string(),
                    name: "Team1".to_string(),
                    organization: "Organization1".to_string(),
                    member_limit: 3,
                },
                profile: Profile {
                    self_introduction: "I'm Alice".to_string(),
                },
                discord_id: "414035444792164353".to_string(),
            },
            Contestant {
                name: "Bob".to_string(),
                display_name: "Bob".to_string(),
                team: Team {
                    code: "2".to_string(),
                    name: "Team2".to_string(),
                    organization: "Organization2".to_string(),
                    member_limit: 3,
                },
                profile: Profile {
                    self_introduction: "I'm Bob".to_string(),
                },
                discord_id: "2345678901".to_string(),
            },
        ])
    }
}
