use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::*;
use serenity::model::prelude::*;

use crate::bot::helpers::HelperError;
use crate::bot::roles;
use crate::bot::Bot;

#[derive(Debug, thiserror::Error)]
enum AskCommandError {
    #[error("質問のタイトルは32文字以内でなければなりません。「問題〇〇の初期条件について」など、簡潔にまとめて再度お試しください。")]
    TitleTooLongError,

    #[error("このコマンドはテキストチャンネル以外から呼び出すことはできません。")]
    InvalidChannelTypeError,

    #[error("予期しないエラーが発生しました。")]
    HelperError(#[from] HelperError),
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
                    .description("質問タイトル（32文字以内）")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    pub async fn handle_ask_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let (guild_channel, title) = match self.validate_ask_command(interaction).await {
            Ok(v) => v,
            Err(err) => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(err.to_string())
                })
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
            self.edit_response(interaction, |data| data.content(err.to_string()))
                .await?;
        }

        Ok(())
    }

    async fn validate_ask_command<'t>(
        &self,
        interaction: &'t ApplicationCommandInteraction,
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

        // 可読性や識別性から、質問タイトルは32文字以内に制限している。
        if title.chars().count() > 32 {
            return Err(AskCommandError::TitleTooLongError);
        }

        Ok((guild_channel, title))
    }

    async fn do_ask_command(
        &self,
        interaction: &ApplicationCommandInteraction,
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

        self.edit_response(interaction, |data| {
            data.content(format!(
                "{} {} 質問内容を入力してください。",
                sender_mention,
                staff_mentions.join(" ")
            ))
        })
        .await?;

        let message = self.get_response(interaction).await?;

        self.create_public_thread(guild_channel, &message, title)
            .await?;

        Ok(())
    }
}
