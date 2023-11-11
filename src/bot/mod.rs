mod channels;
mod commands;
mod helpers;
mod permissions;
mod roles;

use anyhow::Result;
use serenity::client::Client;
use serenity::http::Http;
use serenity::model::prelude::*;
use tokio::sync::RwLock;

use crate::config::Problem;
use crate::config::Team;
use crate::services::redeploy::RedeployNotifier;
use crate::services::redeploy::RedeployService;

pub struct Bot {
    token: String,
    application_id: ApplicationId,
    guild_id: GuildId,
    discord_client: Http,
    infra_password: String,
    teams: Vec<Team>,
    problems: Vec<Problem>,

    redeploy_service: Box<dyn RedeployService + Send + Sync>,
    redeploy_notifiers: Vec<Box<dyn RedeployNotifier + Send + Sync>>,

    role_cache: RwLock<Option<Vec<Role>>>,
}

impl Bot {
    pub fn new(
        token: String,
        application_id: u64,
        guild_id: u64,
        infra_password: String,
        teams: Vec<Team>,
        problems: Vec<Problem>,
        redeploy_service: Box<dyn RedeployService + Send + Sync>,
        redeploy_notifiers: Vec<Box<dyn RedeployNotifier + Send + Sync>>,
    ) -> Self {
        let application_id = ApplicationId(application_id);
        let guild_id = GuildId(guild_id);
        let discord_client = Http::new_with_application_id(&token, application_id.0);
        Bot {
            token,
            application_id,
            guild_id,
            discord_client,
            infra_password,
            teams,
            problems,
            redeploy_service,
            redeploy_notifiers,
            role_cache: RwLock::new(None),
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
