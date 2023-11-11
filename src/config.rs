use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::models::Problem;
use crate::models::Team;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub staff: StaffConfiguration,
    pub discord: DiscordConfiguration,
    pub redeploy: RedeployServiceConfiguration,

    #[serde(default)]
    pub teams: Vec<Team>,

    #[serde(default)]
    pub problems: Vec<Problem>,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Configuration> {
        let file = File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
    }
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
pub struct RedeployServiceConfiguration {
    pub baseurl: String,
    pub username: String,
    pub password: String,

    pub notifiers: RedeployNotifiersConfiguration,
}

#[derive(Debug, Deserialize)]
pub struct RedeployNotifiersConfiguration {
    pub discord: Option<DiscordRedeployNotifierConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct DiscordRedeployNotifierConfiguration {
    pub webhook_url: String,
}
