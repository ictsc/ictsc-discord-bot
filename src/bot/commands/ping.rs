use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

use crate::bot::Bot;

impl Bot {
    pub fn create_ping_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command.name("ping").description("botの生存確認をします。")
    }

    pub async fn handle_ping_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        tracing::trace!("send acknowledgement");
        self.respond(interaction, |data| data.content("pong!"))
            .await?;
        Ok(())
    }
}
