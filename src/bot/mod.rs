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

#[async_trait]
impl EventHandler for Bot {
    #[tracing::instrument(skip_all, fields(
        id = ?guild.id,
        name = ?guild.name,
        owner_id = ?guild.owner_id,
    ))]
    async fn guild_create(&self, _: Context, guild: Guild) {
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
        match interaction {
            Interaction::ApplicationCommand(interaction) => {
                self.handle_application_command(&ctx, &interaction).await
            },
            _ => {},
        };
    }
}
