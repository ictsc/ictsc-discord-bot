use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::InteractionResponseType;
use serenity::prelude::*;

pub struct Command;

impl Command {
    pub fn create<'a>(
        &self,
        command: &'a mut CreateApplicationCommand,
    ) -> &'a mut CreateApplicationCommand {
        command.name("ping").description("A ping command")
    }

    pub async fn run(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        command
            .create_interaction_response(&ctx.http, |resp| {
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("pong!"))
            })
            .await?;

        Ok(())
    }
}
