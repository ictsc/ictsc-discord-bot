use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub discord: DiscordConfiguration,
    pub slack: Option<SlackConfiguration>,

    #[serde(default)]
    pub teams: Vec<TeamConfiguration>,

    #[serde(default)]
    pub problems: Vec<ProblemConfiguration>,
}

impl Configuration {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Configuration> {
        let file = File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
    }
}

impl From<Configuration> for bot::Configuration {
    fn from(config: Configuration) -> Self {
        Self {
            token: config.discord.token,
            guild_id: config.discord.guild_id,
            application_id: config.discord.application_id,
            teams: config.teams.into_iter().map(|team| team.into()).collect(),
            problems: config
                .problems
                .into_iter()
                .map(|prob| prob.into())
                .collect(),
        }
    }
}

impl From<TeamConfiguration> for bot::TeamConfiguration {
    fn from(team: TeamConfiguration) -> Self {
        Self {
            id: team.id,
            name: team.name,
            organization: team.organization,
            channel_name: team.channel_name,
            role_name: team.role_name,
            invitation_code: team.invitation_code,
        }
    }
}

impl From<ProblemConfiguration> for bot::ProblemConfiguration {
    fn from(problem: ProblemConfiguration) -> Self {
        Self {
            id: problem.id,
            code: problem.code,
            name: problem.name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfiguration {
    pub token: String,
    pub application_id: u64,
    pub guild_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct SlackConfiguration {
    pub username: String,
    pub icon_emoji: String,
    pub webhook_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamConfiguration {
    pub id: String,
    pub name: String,
    pub organization: String,
    pub channel_name: String,
    pub role_name: String,
    pub invitation_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ProblemConfiguration {
    pub id: String,
    pub code: String,
    pub name: String,
}
