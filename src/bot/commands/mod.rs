mod archive;
mod ask;
mod join;
mod ping;
mod redeploy;

use std::collections::HashMap;

use anyhow::Result;
use serenity::{all::CreateInvite, client::Context};

use crate::bot::*;

impl Bot {
    pub async fn sync_global_application_commands(&self) -> Result<()> {
        let desired = HashMap::from([
            (String::from("ping"), Bot::create_ping_command()),
            (String::from("join"), Bot::create_join_command()),
        ]);

        let current = self.discord_client.get_global_commands().await?;

        for command in current {
            if !desired.contains_key(&command.name) {
                self.discord_client
                    .delete_global_command(command.id)
                    .await?;
                tracing::debug!(?command, "deleted global command");
            } else if self.disabled_commands.contains(&command.name) {
                self.discord_client
                    .delete_global_command(command.id)
                    .await?;
                tracing::debug!(?command, "deleted disabled global command");
            }
        }

        for (name, builder) in desired {
            if self.disabled_commands.contains(&name) {
                tracing::debug!(command = ?name, "skipping disabled command");
                continue;
            }
            tracing::debug!(command = ?name, "Syncing command");
            Command::create_global_command(&self.discord_client, builder).await?;
        }

        Ok(())
    }

    pub async fn sync_guild_application_commands(&self) -> Result<()> {
        let desired = HashMap::from([
            (String::from("archive"), Bot::create_archive_command()),
            (String::from("ask"), Bot::create_ask_command()),
            (String::from("redeploy"), Bot::create_redeploy_command()),
        ]);

        let current = self
            .discord_client
            .get_guild_commands(self.guild_id)
            .await?;

        for command in current {
            if !desired.contains_key(&command.name) {
                self.discord_client
                    .delete_guild_command(self.guild_id, command.id)
                    .await?;
                tracing::debug!(?command, "deleted guild command");
            } else if self.disabled_commands.contains(&command.name) {
                self.discord_client
                    .delete_guild_command(self.guild_id, command.id)
                    .await?;
                tracing::debug!(?command, "deleted disabled guild command");
            }
        }

        for (name, builder) in desired {
            if self.disabled_commands.contains(&name) {
                tracing::debug!(command = ?name, "skipping disabled command");
                continue;
            }
            tracing::debug!(command = ?name, "Syncing command");
            self.guild_id
                .create_command(&self.discord_client, builder)
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_commands(&self) -> Result<()> {
        tracing::info!("delete global application commands");
        let commands = self.discord_client.get_global_commands().await?;

        for command in commands {
            tracing::debug!(?command, "delete global application command");
            self.discord_client
                .delete_global_command(command.id)
                .await?;
        }

        tracing::info!("delete guild application commands");
        let commands = self
            .discord_client
            .get_guild_commands(self.guild_id)
            .await?;

        for command in commands {
            tracing::debug!(?command, "delete guild application command");
            self.discord_client
                .delete_guild_command(self.guild_id, command.id)
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn create_invites(&self) -> Result<()> {
        // Implementation for creating invites
        let channels = self.guild_id.channels(&self.discord_client).await?;
        for team in &self.teams {
            let role_name = &team.role_name;
            let role_id = self
                .get_roles()
                .await?
                .iter()
                .find(|role| role.name == *role_name)
                .map(|role| role.id);
            if role_id.is_none() {
                tracing::warn!(role_name = ?role_name, "Role for team not found, skipping invite creation");
                continue;
            }
            let role_id = role_id.unwrap();

            let channel_name = format!(
                "{}-{}",
                team.id,
                crate::bot::channels::TEXT_CHANNEL_NAME_SUFFIX
            );
            let create_invite = CreateInvite::new()
                .max_age(86400 * 7) // 7 days
                .max_uses(0)
                .temporary(false)
                .role_ids([role_id]);
            if let Some((_, channel)) = channels
                .iter()
                .find(|(_, channel)| channel.name == channel_name)
            {
                let invite = channel
                    .create_invite(&self.discord_client, create_invite)
                    .await?;
                let url = invite.url();
                println!("{}: {}", team.role_name, url);
            } else {
                tracing::warn!(channel_name = ?channel_name, "Channel for team not found, skipping invite creation");
            }
        }
        Ok(())
    }
}

impl Bot {
    #[tracing::instrument(skip_all, fields(
        id = ?interaction.id,
        guild_id = ?interaction.guild_id,
        channel_id = ?interaction.channel_id,
        user_id = ?interaction.user.id,
        user_name = ?interaction.user.name,
    ))]
    pub async fn handle_application_command(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
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
