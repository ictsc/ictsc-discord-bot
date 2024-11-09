use anyhow::Result;
use serenity::all::CreateCommand;
use serenity::all::EditInteractionResponse;
use serenity::model::prelude::*;

use crate::bot::helpers::HelperError;
use crate::bot::Bot;

#[derive(Debug, thiserror::Error)]
enum ArchiveCommandError {
    #[error("このコマンドは質問スレッド以外から呼び出すことはできません。")]
    ChannelNotThreadError,

    #[error("予期しないエラーが発生しました。")]
    HelperError(#[from] HelperError),
}

type ArchiveCommandResult<T> = std::result::Result<T, ArchiveCommandError>;

impl Bot {
    pub fn create_archive_command() -> CreateCommand {
        CreateCommand::new("archive").description("運営への質問スレッドを終了します")
    }

    pub async fn handle_archive_command(&self, interaction: &CommandInteraction) -> Result<()> {
        tracing::debug!("send acknowledgement");
        self.defer_response(interaction).await?;

        if let Err(err) = self.do_archive_command(interaction).await {
            tracing::error!(?err, "failed to do archive command");
            self.edit_response(
                interaction,
                EditInteractionResponse::new().content(err.to_string()),
            )
            .await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn do_archive_command(
        &self,
        interaction: &CommandInteraction,
    ) -> ArchiveCommandResult<()> {
        let channel_id = interaction.channel_id;
        let channel = self.get_channel(channel_id).await?;

        let mut guild_channel = match channel {
            Channel::Guild(guild_channel) => guild_channel,
            _ => return Err(ArchiveCommandError::ChannelNotThreadError),
        };

        if guild_channel.kind != ChannelType::PublicThread {
            return Err(ArchiveCommandError::ChannelNotThreadError);
        }

        self.archive_thread(&mut guild_channel).await?;

        self.edit_response(
            interaction,
            EditInteractionResponse::new().content("質問スレッドを終了しました。"),
        )
        .await?;

        Ok(())
    }
}
