use crate::*;

use std::collections::HashMap;

use anyhow::Result;
use serenity::async_trait;
use serenity::builder::*;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

type CommandCreator =
    Box<dyn FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand + Send>;
type CommandDefinitions<'a> = HashMap<&'a str, CommandCreator>;

pub struct Bot {
    config: Configuration,
}

pub struct Configuration {
    pub token: String,
    pub application_id: u64,
}

fn setup_global_application_command_definitions() -> CommandDefinitions<'static> {
    let mut definitions = CommandDefinitions::new();

    definitions.insert(
        "join",
        Box::new(|command| {
            command
                .name("join")
                .description("join")
                .create_option(|option| {
                    option
                        .name("invitation_code")
                        .description("招待コード")
                        .kind(ApplicationCommandOptionType::String)
                        .required(true)
                })
        }),
    );

    definitions
}

fn setup_application_command_definitions() -> CommandDefinitions<'static> {
    let mut definitions = CommandDefinitions::new();

    definitions.insert(
        "ping",
        Box::new(|command| command.name("ping").description("botの生存確認をします。")),
    );

    definitions.insert(
        "join",
        Box::new(|command| {
            command
                .name("join")
                .description("join")
                .create_option(|option| {
                    option
                        .name("invitation_code")
                        .description("招待コード")
                        .kind(ApplicationCommandOptionType::String)
                        .required(true)
                })
        }),
    );

    definitions
}

impl Bot {
    pub fn new(config: Configuration) -> Self {
        Bot { config }
    }

    pub async fn start(self) -> Result<()> {
        let token = &self.config.token;
        let application_id = self.config.application_id;

        let mut client = Client::builder(token)
            .application_id(application_id)
            .event_handler(self)
            .await?;

        Ok(client.start().await?)
    }

    async fn setup_global_application_command(&self, ctx: Context) {
        let definitions = setup_global_application_command_definitions();

        let commands = ApplicationCommand::get_global_application_commands(&ctx.http)
            .await
            .unwrap();
        for command in &commands {
            if !definitions.contains_key(command.name.as_str()) {
                log::debug!("delete global application command: {:?}", command);
                ApplicationCommand::delete_global_application_command(&ctx.http, command.id)
                    .await
                    .unwrap();
            }
        }

        for (name, handler) in definitions {
            log::debug!("create global application command: {:?}", name);
            ApplicationCommand::create_global_application_command(&ctx.http, handler)
                .await
                .unwrap();
        }
    }

    async fn setup_application_command(&self, ctx: Context, guild: Guild) {
        let definitions = setup_application_command_definitions();

        let commands = guild.get_application_commands(&ctx.http).await.unwrap();
        for command in &commands {
            if !definitions.contains_key(command.name.as_str()) {
                log::debug!("delete application command: {:?}", command);
                guild
                    .delete_application_command(&ctx.http, command.id)
                    .await
                    .unwrap();
            }
        }

        for (name, handler) in definitions {
            log::debug!("create application command: {:?}", name);
            guild
                .create_application_command(&ctx.http, handler)
                .await
                .unwrap();
        }
    }

    async fn teardown_application_command(&self, ctx: Context, guild: Guild) {
        let commands = guild.get_application_commands(&ctx.http).await.unwrap();

        for command in &commands {
            log::debug!("delete application command: {:?}", command);
            guild
                .delete_application_command(&ctx.http, command.id)
                .await
                .unwrap();
        }
    }

    async fn handle_command_ping(&self, ctx: Context, command: ApplicationCommandInteraction) {
        InteractionHelper::send(ctx, command, "pong!").await;
    }

    async fn handle_application_command(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) {
        match command.data.name.as_str() {
            "ping" => self.handle_command_ping(ctx, command).await,
            _ => {
                log::error!("received command unhandled: {:?}", command);
                InteractionHelper::send_followup(ctx, command, "internal server error").await;
            }
        };
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        log::debug!("called guild_create: {:?}", guild);

        self.setup_application_command(ctx, guild).await;
    }

    async fn ready(&self, _ctx: Context, _: Ready) {
        log::debug!("called ready");
        log::info!("started bot");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log::debug!("called interaction_create: {:?}", interaction);

        match interaction {
            Interaction::ApplicationCommand(command) => {
                self.handle_application_command(ctx, command).await
            }
            _ => {}
        };
    }
}
