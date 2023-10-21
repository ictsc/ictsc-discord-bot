mod ask;
mod archive;
mod join;
mod ping;

use crate::CommandResult;

use super::Bot;
use crate::error::*;
use crate::{InteractionDeferredResponder, InteractionHelper};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;

impl Bot {
    async fn sync_global_application_commands(&self) -> Result<()> {
        tracing::info!("sync global application commands");

        tracing::debug!("sync ping command");
        Command::create_global_application_command(&self.discord_client, Bot::create_ping_command)
            .await?;

        tracing::debug!("sync join command");
        Command::create_global_application_command(&self.discord_client, Bot::create_join_command)
            .await?;

        Ok(())
    }

    async fn sync_guild_application_commands(&self) -> Result<()> {
        tracing::info!("sync guild application commands");

        tracing::debug!("sync ask command");
        self.guild_id
            .create_application_command(&self.discord_client, Bot::create_ask_command)
            .await?;

        tracing::debug!("sync archive command");
        self.guild_id
            .create_application_command(&self.discord_client, Bot::create_archive_command)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_commands(&self) -> CommandResult<()> {
        tracing::info!("delete global application commands");
        let commands = self
            .discord_client
            .get_global_application_commands()
            .await?;

        for command in commands {
            tracing::debug!(?command, "delete global application command");
            self.discord_client
                .delete_global_application_command(command.id.0)
                .await?;
        }

        tracing::info!("delete guild application commands");
        let commands = self
            .discord_client
            .get_guild_application_commands(self.guild_id.0)
            .await?;

        for command in commands {
            tracing::debug!(?command, "delete guild application command");
            self.discord_client
                .delete_guild_application_command(self.guild_id.0, command.id.0)
                .await?;
        }

        Ok(())
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
        tracing::debug!("guild_create called");
        if guild.id != self.guild_id {
            tracing::warn!("target guild is not for contest, skipped");
            return;
        }

        if let Err(err) = self.sync_guild_application_commands().await {
            tracing::error!(?err, "failed to sync guild application commands");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {}

    #[tracing::instrument(skip_all, fields(
        application_id = ?_ready.application.id,
        session_id = ?_ready.session_id,
        user_id = ?_ready.user.id,
        user_name = ?_ready.user.name,
    ))]
    async fn ready(&self, _: Context, _ready: Ready) {
        tracing::info!("bot is ready!");
        if let Err(err) = self.sync_global_application_commands().await {
            tracing::error!(?err, "failed to sync global application commands")
        }
    }

    #[tracing::instrument(skip_all)]
    async fn interaction_create(&self, _: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(interaction) => {
                self.handle_application_command(&interaction).await
            }
            _ => {}
        };
    }
}

impl Bot {
    #[tracing::instrument(skip_all, fields(
        id = ?interaction.id,
        application_id = ?interaction.application_id,
        kind = ?interaction.kind,
        guild_id = ?interaction.guild_id,
        channel_id = ?interaction.channel_id,
        user_id = ?interaction.user.id,
        user_name = ?interaction.user.name,
    ))]
    async fn handle_application_command(&self, interaction: &ApplicationCommandInteraction) {
        let name = interaction.data.name.as_str();

        let result = match name {
            "ask" => self.handle_ask_command(interaction).await,
            "archive" => self.handle_archive_command(interaction).await,
            "join" => self.handle_join_command(interaction).await,
            "ping" => self.handle_ping_command(interaction).await,
            _ => Ok(()),
        };

        if let Err(err) = result {
            tracing::error!(?err, "failed to handle application command");
            let _ = InteractionHelper::defer_respond(
                &self.discord_client,
                interaction,
                format!("エラーが発生しました: {}", err),
            )
            .await;
        };
    }
}
