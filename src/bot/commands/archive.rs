use super::Bot;
use crate::*;

use crate::InteractionHelper;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;

impl Bot {
    pub fn create_archive_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name("archive")
            .description("運営への質問スレッドを終了します")
    }

    pub async fn handle_archive_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let channel_id = interaction.channel_id;
        let channel = self.discord_client.get_channel(channel_id.0).await?;

        let guild_channel = match channel {
            Channel::Guild(guild_channel) => guild_channel,
            _ => {
                interaction.create_interaction_response(&self.discord_client, |response| {
                    response.kind(InteractionResponseType::ChannelMessageWithSource);
                    response.interaction_response_data(|data| {
                        data.ephemeral(true).content("このコマンドはスレッド内でのみ使用できます。")
                    })
                }).await?;
                return Ok(());
            }
        };

        if guild_channel.kind != ChannelType::PublicThread {
            interaction.create_interaction_response(&self.discord_client, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(|data| {
                    data.ephemeral(true).content("このコマンドはスレッド内でのみ使用できます。")
                })
            }).await?;
            return Ok(());
        }

        tracing::trace!("send acknowledgement");
        let _ = InteractionHelper::defer(&self.discord_client, interaction).await;

        channel_id.edit_thread(&self.discord_client, |thread| {
            thread.archived(true)
        }).await?;

        InteractionHelper::defer_respond(&self.discord_client, interaction, "質問スレッドを終了しました。").await?;

        Ok(())
    }
}
