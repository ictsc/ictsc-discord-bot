use super::Bot;
use crate::*;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::*;
use serenity::model::prelude::*;

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
        let title = self.get_option_as_str(interaction, "title").unwrap();

        // TODO: スレッドタイトルの長さチェック

        let channel_id = interaction.channel_id;
        let channel = channel_id.to_channel(&self.discord_client).await?;
        match channel {
            Channel::Guild(channel) => {
                if channel.kind == ChannelType::PublicThread {
                    self.reply(interaction, |data| {
                        data.ephemeral(true)
                            .content("質問スレッド内でこのコマンドを使用することはできません。")
                    })
                    .await?;
                    return Ok(());
                }
            }
            _ => {
                self.reply(interaction, |data| {
                    data.ephemeral(true)
                        .content("このコマンドはサーバ内でのみ使用できます。")
                })
                .await?;
                return Ok(());
            }
        };

        tracing::trace!("send acknowledgement");
        self.defer_reply(interaction).await?;

        let sender = &interaction.user;
        let staff_roles = self
            .find_roles_by_name(bot::roles::STAFF_ROLE_NAME)
            .await
            .map_err(|err| SystemError::UnexpectedError(err.to_string()))?;

        let sender_mention = Mention::from(sender.id).to_string();
        let staff_mensions: Vec<_> = staff_roles
            .into_iter()
            .map(|role| Mention::from(role.id).to_string())
            .collect();

        self.edit_reply(interaction, |data| {
            data.content(format!(
                "{} {} 質問内容を入力してください。",
                sender_mention,
                staff_mensions.join(" ")
            ))
        })
        .await?;

        let message = self.get_response_message(interaction).await?;
        channel_id
            .create_public_thread(&self.discord_client, message.id, |thread| {
                thread.name(title)
            })
            .await?;

        Ok(())
    }
}
