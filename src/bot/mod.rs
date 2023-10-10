mod channels;
mod commands;
mod roles;

use anyhow::Result;
use serenity::client::Client;
use serenity::client::EventHandler;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::model::prelude::*;

pub struct Bot {
    token: String,
    application_id: ApplicationId,
    guild_id: GuildId,
    teams: Vec<Team>,
    discord_client: Http,
}

pub struct Team {
    pub role_name: String,
}

impl Bot {
    pub fn new(token: String, application_id: u64, guild_id: u64, teams: Vec<Team>) -> Self {
        let application_id = ApplicationId(application_id);
        let guild_id = GuildId(guild_id);
        let discord_client = Http::new_with_application_id(&token, application_id.0);
        Bot {
            token,
            application_id,
            guild_id,
            teams,
            discord_client,
        }
    }

    pub async fn start(self) -> Result<()> {
        let token = &self.token;
        let application_id = self.application_id;

        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::DIRECT_MESSAGES;

        let mut client = Client::builder(token, intents)
            .application_id(application_id.0)
            .event_handler(self)
            .await?;

        Ok(client.start().await?)
    }
}
