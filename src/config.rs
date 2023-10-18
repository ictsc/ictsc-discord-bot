use std::fs::File;
use std::path::Path;

use anyhow::Result;
use bot::Team;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub staff: StaffConfiguration,
    pub discord: DiscordConfiguration,
    pub slack: Option<SlackConfiguration>,
    pub recreate: RecreateServiceConfiguration,

    #[serde(default)]
    pub teams: Vec<TeamConfiguration>,

    #[serde(default)]
    pub problems: Vec<ProblemConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct StaffConfiguration {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfiguration {
    pub token: String,
    pub application_id: u64,
    pub guild_id: u64,
    pub disabled_commands: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SlackConfiguration {
    pub username: String,
    pub icon_emoji: String,
    pub webhook_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RecreateServiceConfiguration {
    pub baseurl: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamConfiguration {
    // pub id: String,
    // pub category_name: String,
    pub role_name: String,
    pub invitation_code: String,
    // pub user_group_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProblemConfiguration {
    pub id: String,
    pub name: String,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Configuration> {
        let file = File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
    }

    pub fn teams(&self) -> Vec<Team> {
        self.teams
            .iter()
            .map(|c| Team {
                role_name: c.role_name.clone(),
                invitation_code: c.invitation_code.clone(),
            })
            .collect()
    }
}
