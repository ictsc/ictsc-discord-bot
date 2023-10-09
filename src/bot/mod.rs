mod commands;

use crate::errors::*;

use serenity::client::Client;
use serenity::client::EventHandler;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::model::prelude::*;

pub struct Bot {
    token: String,
    application_id: u64,
    guild_id: u64,
    discord_client: Http,
}

impl Bot {
    pub fn new(token: String, application_id: u64, guild_id: u64) -> Self {
        let discord_client = Http::new_with_application_id(&token, application_id);
        Bot {
            token,
            application_id,
            guild_id,
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
            .application_id(application_id)
            .event_handler(self)
            .await?;

        Ok(client.start().await?)
    }
}
