use anyhow::Result;
use serenity::all::CommandInteraction;
use serenity::all::CreateInteractionResponseMessage;
use serenity::builder::CreateCommand;

use crate::bot::Bot;

impl Bot {
    pub fn create_ping_command() -> CreateCommand {
        CreateCommand::new("ping").description("botの生存確認をします。")
    }

    #[tracing::instrument(skip_all)]
    pub async fn handle_ping_command(&self, interaction: &CommandInteraction) -> Result<()> {
        tracing::trace!("send acknowledgement");
        self.respond(
            interaction,
            CreateInteractionResponseMessage::new().content("pong!"),
        )
        .await?;
        Ok(())
    }
}
