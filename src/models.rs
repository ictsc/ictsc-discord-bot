use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub id: String,
    pub role_name: String,
    pub invitation_code: String,
    pub user_group_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Problem {
    pub code: String,
    pub name: String,
}
