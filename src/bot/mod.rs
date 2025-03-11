mod channels;
mod commands;
mod helpers;
mod permissions;
mod roles;

use anyhow::Result;
use async_trait::async_trait;
use serenity::client::Client;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tokio::sync::RwLock;

use crate::models::Problem;
use crate::models::Team;
use crate::services::contestant::ContestantService;
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
    contestant_service: Box<dyn ContestantService + Send + Sync>,

    configure_channel_topics: bool,

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
        contestant_service: Box<dyn ContestantService + Send + Sync>,
        configure_channel_topics: bool,
    ) -> Self {
        let application_id = ApplicationId::new(application_id);
        let guild_id = GuildId::new(guild_id);
        let discord_client = Http::new(&token);
        discord_client.set_application_id(application_id);
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
            contestant_service,
            configure_channel_topics,
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
            .application_id(application_id)
            .event_handler(self)
            .await?;

        Ok(client.start().await?)
    }
}

#[async_trait]
impl EventHandler for Bot {
    #[tracing::instrument(skip_all, fields(
        id = ?guild.id,
        name = ?guild.name,
        owner_id = ?guild.owner_id,
    ))]
    async fn guild_create(&self, _: Context, guild: Guild, _: Option<bool>) {
        if guild.id != self.guild_id {
            tracing::info!("Target guild is not for contest, skipping");
            return;
        }

        tracing::info!("Updating role cache");
        if let Err(err) = self.update_role_cache().await {
            tracing::error!(?err, "failed to update role cache");
        }

        tracing::info!("Syncing guild application commands");
        if let Err(err) = self.sync_guild_application_commands().await {
            tracing::error!(?err, "failed to sync guild application commands");
        }
    }

    #[tracing::instrument(skip_all, fields(
        application_id = ?_ready.application.id,
        session_id = ?_ready.session_id,
        user_id = ?_ready.user.id,
        user_name = ?_ready.user.name,
    ))]
    async fn ready(&self, _: Context, _ready: Ready) {
        tracing::info!("Syncing global application commands");
        if let Err(err) = self.sync_global_application_commands().await {
            tracing::error!(?err, "failed to sync global application commands")
        }
    }

    #[tracing::instrument(skip_all)]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(interaction) = interaction {
            self.handle_application_command(&ctx, &interaction).await
        };
    }
}
