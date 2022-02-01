use std::any::{Any, TypeId};
use crate::*;

use std::collections::HashMap;

use crate::commands::whoami::WhoAmICommand;
use anyhow::Result;
use serenity::async_trait;
use serenity::builder::*;
use serenity::http::Http;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::commands::ask::AskCommand;

type CommandCreator =
    Box<dyn FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand + Send>;
type CommandDefinitions<'a> = HashMap<&'a str, CommandCreator>;

pub struct Bot {
    config: Configuration,
}

pub struct Configuration {
    pub token: String,
    pub guild_id: u64,
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
        "whoami",
        Box::new(|command| {
            command
                .name("whoami")
                .description("ユーザ情報を表示します（デバッグ用）")
        }),
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

    definitions.insert(
        "ask",
        Box::new(|command| {
            command
                .name("ask")
                .description("運営への質問スレッドを開始します")
                .create_option(|option| {
                    option
                        .name("summary")
                        .description("質問内容の簡潔な説明（例：問題〇〇について, ...）")
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

    pub async fn create_admin_role(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        RoleManager.sync(http, guild_id, CreateRoleInput {
            name: String::from("ICTSC2021 Staff"),
            color: 14942278,
            hoist: true,
            mentionable: true,
            permissions: Permissions::all(),
        }).await;

        Ok(())
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

    async fn handle_command_ping(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        InteractionHelper::send(&ctx.http, command, "pong!").await
    }

    async fn handle_command_whoami(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        let handler = WhoAmICommand::new(UserManager);

        let user_id = command.user.id;
        let result = handler.run(&ctx.http, user_id).await;

        match result {
            Ok(info) => {
                InteractionHelper::send_table(&ctx.http, command, info).await
            }
            Err(reason) => {
                log::error!("failed to run whoami: {:?}", reason);
                InteractionHelper::send_ephemeral(&ctx.http, command, "internal server error").await
            }
        }
    }

    async fn handle_command_ask(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        let handler = AskCommand::new(UserManager, ThreadManager);

        let channel_id = command.channel_id;
        // TODO: validate channel type

        let user_id = command.user.id;

        let summary = InteractionHelper::value_of_as_str(&command, "summary").unwrap();

        let result = handler.run(&ctx.http, channel_id, user_id, summary).await;

        match result {
            Ok(_) => {
                InteractionHelper::send_ephemeral(&ctx.http, command, "質問スレッドが開始されました。").await
            }
            Err(reason) => {
                log::error!("failed to run ask: {:?}", reason);
                InteractionHelper::send_ephemeral(&ctx.http, command, "internal server error").await
            }
        }
    }

    async fn handle_application_command(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) {
        let name = command.data.name.clone();

        let result = match name.as_str() {
            "ping" => self.handle_command_ping(ctx, command).await,
            "whoami" => self.handle_command_whoami(ctx, command).await,
            "ask" => self.handle_command_ask(ctx, command).await,
            _ => {
                log::error!("received command unhandled: {:?}", command);
                InteractionHelper::send_ephemeral(&ctx.http, command, "internal server error").await
            }
        };

        match result {
            Ok(_) => (),
            Err(reason) => {
                log::error!("failed to handle application command: (name: {}, reason: {:?})", name, reason);
            }
        }
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
