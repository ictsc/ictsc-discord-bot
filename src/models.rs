use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Team {
    #[validate(custom(function = "validate_team_id"))]
    pub id: String,
    pub role_name: String,
    pub invitation_code: String,
}

// Custom validation function for team ID
// Normally, discord can accept more wider range of characters (e.g. unicode).
// However, it's a little hard to track discord specifications.
fn validate_team_id(id: &str) -> Result<(), ValidationError> {
    if id.chars().all(|c| c.is_lowercase() || c.is_digit(10)) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "id must be lowercase alphanumeric characters",
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Problem {
    pub code: String,
    pub name: String,
}
