mod archive;
mod ask;
mod join;
mod ping;
mod redeploy;

use anyhow::Result;
use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

use crate::bot::*;

impl Bot {
    pub async fn sync_global_application_commands(&self) -> Result<()> {
        tracing::debug!("Syncing ping command");
        Command::create_global_application_command(&self.discord_client, Bot::create_ping_command)
            .await?;

        tracing::debug!("Syncing join command");
        Command::create_global_application_command(&self.discord_client, Bot::create_join_command)
            .await?;

        Ok(())
    }

    pub async fn sync_guild_application_commands(&self) -> Result<()> {
        tracing::debug!("sync archive command");
        self.guild_id
            .create_application_command(&self.discord_client, Bot::create_archive_command)
            .await?;

        tracing::debug!("sync ask command");
        self.guild_id
            .create_application_command(&self.discord_client, Bot::create_ask_command)
            .await?;

        tracing::debug!("sync redeploy command");
        self.guild_id
            .create_application_command(&self.discord_client, Bot::create_redeploy_command)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_commands(&self) -> Result<()> {
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
    pub async fn handle_application_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) {
        let name = interaction.data.name.as_str();

        let result = match name {
            "archive" => self.handle_archive_command(interaction).await,
            "ask" => self.handle_ask_command(interaction).await,
            "join" => self.handle_join_command(interaction).await,
            "ping" => self.handle_ping_command(interaction).await,
            "redeploy" => self.handle_redeploy_command(ctx, interaction).await,
            _ => Err(anyhow::anyhow!("unknown command: {}", name)),
        };

        if let Err(err) = result {
            tracing::error!(?err, "failed to handle application command");
        };
    }
}
