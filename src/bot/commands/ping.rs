use super::Bot;
use crate::error::*;

use crate::{InteractionDeferredResponder, InteractionHelper};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

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
        InteractionHelper::defer_respond(&self.discord_client, &interaction, "pong!").await?;
        Ok(())
    }
}
