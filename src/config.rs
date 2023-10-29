use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Team {
    pub role_name: String,
    pub category_name: String,
    pub invitation_code: String,
}

#[derive(Debug, Deserialize)]
pub struct Problem {
    pub code: String,
    pub name: String,
}


#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub staff: StaffConfiguration,
    pub discord: DiscordConfiguration,
    pub recreate: RecreateServiceConfiguration,

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

    pub fn teams(&self) -> Vec<Team> {
        self.teams
            .iter()
            .map(|c| Team {
                role_name: c.role_name.clone(),
                category_name: c.category_name.clone(),
                invitation_code: c.invitation_code.clone(),
            })
            .collect()
    }

    pub fn problems(&self) -> Vec<Problem> {
        self.problems
            .iter()
            .map(|c| Problem {
                code: c.code.clone(),
                name: c.name.clone(),
            })
            .collect()
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
pub struct RecreateServiceConfiguration {
    pub baseurl: String,
    pub username: String,
    pub password: String,
}
