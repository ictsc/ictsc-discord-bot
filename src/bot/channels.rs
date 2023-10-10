use crate::CommandResult;

use super::Bot;

impl Bot {
    #[tracing::instrument(skip_all)]
    pub async fn delete_channels(&self) -> CommandResult<()> {
        tracing::info!("delete all channels");
        for (channel_id, channel) in self.guild_id.channels(&self.discord_client).await? {
            tracing::debug!(?channel, "delete channel");
            channel_id.delete(&self.discord_client).await?;
        }
        Ok(())
    }
}
