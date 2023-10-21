use super::Bot;
use crate::*;

use crate::{InteractionArgumentExtractor, InteractionHelper};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, ResolvedTarget};
use serenity::model::prelude::*;
use serenity::model::prelude::command::*;

impl Bot {
    pub fn create_ask_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name("ask")
            .description("運営への質問スレッドを開始します")
            .create_option(|option| {
                option
                    .name("title")
                    .description("質問タイトル")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    pub async fn handle_ask_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let title =
            InteractionHelper::value_of_as_str(interaction, "title").unwrap();

        let channel_id = interaction.channel_id;
        let message = interaction.get_interaction_response(&self.discord_client).await?;

        let channel = channel_id.to_channel(&self.discord_client).await?;
        match channel {
            Channel::Guild(channel) => {
                if channel.kind == ChannelType::PublicThread || channel.kind == ChannelType::PrivateThread {
                    interaction.create_followup_message(&self.discord_client, |message| {
                        message.content("aaa").ephemeral(true)
                    }).await?;
                    return Ok(())
                }
            },
            _ => {
                InteractionHelper::defer_respond(
                    &self.discord_client, 
                    interaction, 
                    "このコマンドはサーバー内でのみ使用できます。"
                ).await?;
                return Ok(())
            }
        };

        let sender = &interaction.user;
        let staff_roles = self
            .find_roles_by_name(bot::roles::STAFF_ROLE_NAME)
            .await
            .map_err(|err| SystemError::UnexpectedError(err.to_string()))?;

        // TODO: スレッドタイトルの長さチェック

        let sender_mention = Mention::from(sender.id).to_string();
        let staff_mensions: Vec<_> = staff_roles
            .into_iter()
            .map(|role| Mention::from(role.id).to_string())
            .collect();

        InteractionHelper::defer_respond(&self.discord_client, interaction, 
            format!("{} {} 質問内容を入力してください。", sender_mention, staff_mensions.join(" "))).await?;

        channel_id.create_public_thread(&self.discord_client, message.id, |thread| {
            thread.name(title)
        }).await?;

        Ok(())
    }
}
