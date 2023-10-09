use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub staff: StaffConfiguration,
    pub discord: DiscordConfiguration,
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
pub struct RecreateServiceConfiguration {
    pub baseurl: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamConfiguration {
    pub id: String,
    pub channel_name: String,
    pub role_name: String,
    pub invitation_code: String,
    pub user_group_id: String,
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
}

impl From<Configuration> for bot::Configuration {
    fn from(config: Configuration) -> Self {
        Self {
            token: config.discord.token,
            guild_id: config.discord.guild_id,
            application_id: config.discord.application_id,
            disabled_commands: config.discord.disabled_commands.unwrap_or_default(),
            staff: config.staff.into(),
            recreate_service: config.recreate.into(),
            teams: config.teams.into_iter().map(|team| team.into()).collect(),
            problems: config
                .problems
                .into_iter()
                .map(|prob| prob.into())
                .collect(),
        }
    }
}

impl From<StaffConfiguration> for bot::StaffConfiguration {
    fn from(config: StaffConfiguration) -> Self {
        Self {
            password: config.password,
        }
    }
}

impl From<RecreateServiceConfiguration> for bot::RecreateServiceConfiguration {
    fn from(config: RecreateServiceConfiguration) -> Self {
        Self {
            baseurl: config.baseurl,
            username: config.username,
            password: config.password,
        }
    }
}

impl From<TeamConfiguration> for bot::TeamConfiguration {
    fn from(team: TeamConfiguration) -> Self {
        Self {
            id: team.id,
            channel_name: team.channel_name,
            role_name: team.role_name,
            invitation_code: team.invitation_code,
            user_group_id: team.user_group_id,
        }
    }
}

impl From<ProblemConfiguration> for bot::ProblemConfiguration {
    fn from(problem: ProblemConfiguration) -> Self {
        Self {
            id: problem.id,
            name: problem.name,
        }
    }
}
