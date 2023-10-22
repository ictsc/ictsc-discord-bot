use super::Bot;
use crate::*;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::*;
use serenity::model::prelude::*;

#[derive(Debug, thiserror::Error)]
enum AskCommandError {
    #[error("質問のタイトルは32文字以内でなければなりません。「問題〇〇の初期条件について」など、簡潔にまとめて再度お試しください。")]
    TitleTooLongError,

    #[error("このコマンドはテキストチャンネル以外から呼び出すことはできません。")]
    InvalidChannelTypeError,

    #[error("予期しないエラーが発生しました。")]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

type AskCommandResult<T> = std::result::Result<T, AskCommandError>;

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

    async fn validate_ask_command<'t>(
        &self,
        interaction: &'t ApplicationCommandInteraction,
    ) -> AskCommandResult<&'t str> {
        let channel = interaction
            .channel_id
            .to_channel(&self.discord_client)
            .await
            .map_err(|err| AskCommandError::Error(err.into()))?;

        match channel {
            Channel::Guild(channel) => {
                // Textチャンネル以外ではスレッドは作成できないので、エラーを返す。
                if channel.kind != ChannelType::Text {
                    return Err(AskCommandError::InvalidChannelTypeError);
                }
                channel
            }
            _ => return Err(AskCommandError::InvalidChannelTypeError),
        };

        let title = self.get_option_as_str(interaction, "title").unwrap();

        // 可読性や識別性から、質問タイトルは32文字以内に制限している。
        if title.chars().count() > 32 {
            return Err(AskCommandError::TitleTooLongError);
        }

        Ok(title)
    }

    async fn do_ask_command(
        &self,
        interaction: &ApplicationCommandInteraction,
        title: &str,
    ) -> AskCommandResult<()> {
        let sender = &interaction.user;
        let sender_mention = Mention::from(sender.id).to_string();

        let staff_roles = self
            .find_roles_by_name(bot::roles::STAFF_ROLE_NAME)
            .await
            .map_err(|err| AskCommandError::Error(err.into()))?;

        let staff_mensions: Vec<_> = staff_roles
            .into_iter()
            .map(|role| Mention::from(role.id).to_string())
            .collect();

        self.edit_response(interaction, |data| {
            data.content(format!(
                "{} {} 質問内容を入力してください。",
                sender_mention,
                staff_mensions.join(" ")
            ))
        })
        .await
        .map_err(|err| AskCommandError::Error(err.into()))?;

        let message = self
            .get_response(interaction)
            .await
            .map_err(|err| AskCommandError::Error(err.into()))?;

        interaction
            .channel_id
            .create_public_thread(&self.discord_client, message.id, |thread| {
                thread.name(title)
            })
            .await
            .map_err(|err| AskCommandError::Error(err.into()))?;

        Ok(())
    }

    pub async fn handle_ask_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let title = match self.validate_ask_command(interaction).await {
            Ok(v) => v,
            Err(err) => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(err.to_string())
                })
                .await?;
                return Ok(());
            }
        };

        tracing::trace!("send acknowledgement");
        self.defer_response(interaction).await?;

        if let Err(err) = self.do_ask_command(interaction, title).await {
            tracing::error!(?err, "failed to do ask command");
            self.edit_response(interaction, |data| data.content(err.to_string()))
                .await?;
        }

        Ok(())
    }
}
