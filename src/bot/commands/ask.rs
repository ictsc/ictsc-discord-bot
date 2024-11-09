use anyhow::Result;
use serenity::all::CreateCommand;
use serenity::all::CreateCommandOption;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::CreateMessage;
use serenity::all::EditInteractionResponse;
use serenity::model::prelude::*;

use crate::bot::helpers::HelperError;
use crate::bot::roles;
use crate::bot::Bot;

#[derive(Debug, thiserror::Error)]
enum AskCommandError {
    #[error("質問のタイトルは50文字以内でなければなりません。「問題〇〇の初期条件について」など、簡潔にまとめて再度お試しください。")]
    TitleTooLongError,

    #[error("このコマンドはテキストチャンネル以外から呼び出すことはできません。")]
    InvalidChannelTypeError,

    #[error("予期しないエラーが発生しました。")]
    HelperError(#[from] HelperError),

    #[error("予期しないエラーが発生しました。")]
    Serenity(#[from] serenity::Error),
}

type AskCommandResult<T> = std::result::Result<T, AskCommandError>;

impl Bot {
    pub fn create_ask_command() -> CreateCommand {
        CreateCommand::new("ask")
            .description("運営への質問スレッドを開始します")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "title",
                    "質問タイトル（50文字以内）",
                )
                .required(true),
            )
    }

    #[tracing::instrument(skip_all)]
    pub async fn handle_ask_command(&self, interaction: &CommandInteraction) -> Result<()> {
        let (guild_channel, title) = match self.validate_ask_command(interaction).await {
            Ok(v) => v,
            Err(err) => {
                self.respond(
                    interaction,
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content(err.to_string()),
                )
                .await?;
                return Ok(());
            },
        };

        tracing::debug!("send acknowledgement");
        self.defer_response(interaction).await?;

        if let Err(err) = self
            .do_ask_command(interaction, &guild_channel, title)
            .await
        {
            tracing::error!(?err, "failed to do ask command");
            self.edit_response(
                interaction,
                EditInteractionResponse::new().content(err.to_string()),
            )
            .await?;
        }

        Ok(())
    }

    async fn validate_ask_command<'t>(
        &self,
        interaction: &'t CommandInteraction,
    ) -> AskCommandResult<(GuildChannel, &'t str)> {
        let channel = self.get_channel(interaction.channel_id).await?;

        let guild_channel = match channel {
            Channel::Guild(channel) => {
                // Textチャンネル以外ではスレッドは作成できないので、エラーを返す。
                if channel.kind != ChannelType::Text {
                    return Err(AskCommandError::InvalidChannelTypeError);
                }
                channel
            },
            _ => return Err(AskCommandError::InvalidChannelTypeError),
        };

        let title = self
            .get_option_as_str(&interaction.data.options, "title")
            .unwrap();

        // 可読性や識別性から、質問タイトルは50文字以内に制限している。
        if title.chars().count() > 50 {
            return Err(AskCommandError::TitleTooLongError);
        }

        Ok((guild_channel, title))
    }

    async fn do_ask_command(
        &self,
        interaction: &CommandInteraction,
        guild_channel: &GuildChannel,
        title: &str,
    ) -> AskCommandResult<()> {
        let sender = &interaction.user;
        let sender_mention = Mention::from(sender.id).to_string();

        let staff_mentions: Vec<_> = self
            .find_roles_by_name_cached(roles::STAFF_ROLE_NAME)
            .await?
            .iter()
            .map(|role| Mention::from(role.id).to_string())
            .collect();

        self.edit_response(
            interaction,
            EditInteractionResponse::new()
                .content(format!("{} 質問スレッドを開始します。", sender_mention)),
        )
        .await?;

        let message = self.get_response(interaction).await?;

        let channel = self
            .create_public_thread(guild_channel, &message, title)
            .await?;

        // TODO: 直接メッセージを送信するな！！！
        channel
            .send_message(
                &self.discord_client,
                CreateMessage::new().content(format!(
                    "{} 質問スレッドを開始します。",
                    staff_mentions.join(" ")
                )),
            )
            .await?;

        Ok(())
    }
}
