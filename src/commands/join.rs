use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::guild::Guild;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::InteractionResponseType;
use serenity::prelude::*;

pub struct Command {
    guild: Guild,
}

impl Command {
    pub fn new(guild: Guild) -> Self {
        Self { guild }
    }

    pub fn create<'a>(
        &self,
        command: &'a mut CreateApplicationCommand,
    ) -> &'a mut CreateApplicationCommand {
        command
            .name("join")
            .description("招待コードに紐付けられたチームに参加します。")
            .create_option(|option| {
                option
                    .name("invitation_code")
                    .description("招待コード")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            })
    }

    fn value_of<'a>(&self, command: &'a ApplicationCommandInteraction, name: &str) -> Option<&'a str> {
        for option in &command.data.options {
            if option.name == name {
                return option.value.as_ref().and_then(|v| v.as_str());
            }
        }
        None
    }

    pub async fn run(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        let code = self.value_of(&command, "invitation_code")
            .ok_or(anyhow::anyhow!("missing required parameter"))?;

        command
            .create_interaction_response(&ctx.http, |resp| {
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(code))
            })
            .await?;

        Ok(())
    }
}
