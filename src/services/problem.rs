use crate::models::Problem;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum ProblemError {
    #[error("Problem not found")]
    NotFound,
    #[error("unexpected error, {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[async_trait]
pub trait ProblemService {
    async fn get_problems(&self) -> Result<Vec<Problem>, ProblemError>;
    async fn get_problem(&self, problem_code: &str) -> Result<Problem, ProblemError> {
        self.get_problems()
            .await?
            .into_iter()
            .find(|p| p.code == problem_code)
            .ok_or(ProblemError::NotFound)
    }
}

pub struct StaticProblemService {
    problems: Vec<Problem>,
}

impl StaticProblemService {
    pub fn new(problems: Vec<Problem>) -> Self {
        Self { problems }
    }
}

#[async_trait]
impl ProblemService for StaticProblemService {
    async fn get_problems(&self) -> Result<Vec<Problem>, ProblemError> {
        Ok(self.problems.clone())
    }
}
