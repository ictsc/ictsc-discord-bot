use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: String,
    pub role_name: String,
    pub invitation_code: String,
}

#[derive(Debug, Deserialize)]
pub struct Problem {
    pub code: String,
    pub name: String,
}
