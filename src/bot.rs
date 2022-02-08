use crate::*;

use std::collections::HashMap;

use crate::commands::ask::AskCommand;
use crate::commands::join::JoinCommand;
use crate::commands::whoami::WhoAmICommand;
use crate::commands::ApplicationCommandContext;
use anyhow::Result;
use serenity::async_trait;
use serenity::builder::*;
use serenity::http::Http;
use serenity::model::guild::Target::Role;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::commands::recreate::RecreateCommand;

type CommandCreator =
    Box<dyn FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand + Send>;
type CommandDefinitions<'a> = HashMap<&'a str, CommandCreator>;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub token: String,
    pub guild_id: u64,
    pub application_id: u64,
    pub teams: Vec<TeamConfiguration>,
    pub problems: Vec<ProblemConfiguration>,
}

#[derive(Debug, Clone)]
pub struct TeamConfiguration {
    pub id: String,
    pub name: String,
    pub organization: String,
    pub channel_name: String,
    pub role_name: String,
    pub invitation_code: String,
}

#[derive(Debug, Clone)]
pub struct ProblemConfiguration {
    pub id: String,
    pub code: String,
    pub name: String,
}

pub struct Bot {
    config: Configuration,
    ask_command: AskCommand<UserManager, ThreadManager>,
    join_command: JoinCommand<RoleManager>,
    recreate_command: RecreateCommand<RoleManager>,
    whoami_command: WhoAmICommand<UserManager>,
}

impl Bot {
    pub fn new(config: Configuration) -> Self {
        let guild_id = GuildId(config.guild_id);

        let ask_command = AskCommand::new(UserManager, ThreadManager);
        let join_command = JoinCommand::new(RoleManager, guild_id, &config.teams);
        let recreate_command = RecreateCommand::new(RoleManager, &config.teams, &config.problems);
        let whoami_command = WhoAmICommand::new(UserManager);

        Bot {
            config,
            ask_command,
            join_command,
            recreate_command,
            whoami_command,
        }
    }
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

    definitions.insert(
        "recreate",
        Box::new(|command| {
            command
                .name("recreate")
                .description("問題環境を再作成します。")
                .create_option(|option| {
                    option
                        .name("problem_code")
                        .description("問題コード")
                        .kind(ApplicationCommandOptionType::String)
                        .required(true)
                })
        }),
    );

    definitions
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
                self.handle_application_command(ApplicationCommandContext {
                    context: ctx,
                    command,
                })
                .await;
            }
            _ => {}
        };
    }

    async fn reaction_add(&self, _ctx: Context, reaction: Reaction) {
        log::debug!("called reaction_add: {:?}", reaction);
    }
}

impl Bot {
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

        RoleManager
            .sync(
                http,
                guild_id,
                CreateRoleInput {
                    name: String::from("ICTSC2021 Staff"),
                    color: 14942278,
                    hoist: true,
                    mentionable: true,
                    permissions: Permissions::all(),
                },
            )
            .await?;

        Ok(())
    }

    pub async fn create_admin_channels(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        let category = CategoryChannelManager
            .sync(
                http,
                guild_id,
                CreateCategoryChannelInput {
                    name: String::from("admin"),
                },
            )
            .await?;

        TextChannelManager
            .sync(
                http,
                guild_id,
                CreateTextChannelInput {
                    name: String::from("admin"),
                    category_id: Some(category.id),
                },
            )
            .await?;

        Ok(())
    }

    pub async fn create_team_role(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        for team in &self.config.teams {
            RoleManager
                .sync(
                    http,
                    guild_id,
                    CreateRoleInput {
                        name: team.role_name.clone(),
                        color: 0,
                        hoist: true,
                        mentionable: true,
                        permissions: Permissions::empty(),
                    },
                )
                .await?;
        }

        Ok(())
    }

    pub async fn create_team_channels(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        for team in &self.config.teams {
            let category = CategoryChannelManager
                .sync(
                    http,
                    guild_id,
                    CreateCategoryChannelInput {
                        name: team.channel_name.clone(),
                    },
                )
                .await?;

            TextChannelManager
                .sync(
                    http,
                    guild_id,
                    CreateTextChannelInput {
                        name: team.channel_name.clone(),
                        category_id: Some(category.id),
                    },
                )
                .await?;
        }

        Ok(())
    }
}

impl Bot {
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
}

impl Bot {
    async fn handle_command_ping(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        Ok(InteractionHelper::send(&ctx.context.http, &ctx.command, "pong!").await?)
    }

    async fn handle_command_ask(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let summary = InteractionHelper::value_of_as_str(&ctx.command, "summary").unwrap();
        Ok(self.ask_command.run(ctx, summary.into()).await?)
    }

    async fn handle_command_whoami(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        Ok(self.whoami_command.run(ctx).await?)
    }

    async fn handle_command_join(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let invitation_code =
            InteractionHelper::value_of_as_str(&ctx.command, "invitation_code").unwrap();
        Ok(self.join_command.run(ctx, invitation_code.into()).await?)
    }

    async fn handle_command_recreate(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let problem_code =
            InteractionHelper::value_of_as_str(&ctx.command, "problem_code").unwrap();
        Ok(self.recreate_command.run(ctx, problem_code.into()).await?)
    }

    async fn handle_application_command(&self, ctx: ApplicationCommandContext) {
        let name = ctx.command.data.name.as_str();

        let result = match name {
            "ping" => self.handle_command_ping(&ctx).await,
            "whoami" => self.handle_command_whoami(&ctx).await,
            "ask" => self.handle_command_ask(&ctx).await,
            "join" => self.handle_command_join(&ctx).await,
            "recreate" => self.handle_command_recreate(&ctx).await,
            _ => {
                log::error!("received command unhandled: {:?}", ctx.command);
                InteractionHelper::send_ephemeral(
                    &ctx.context.http,
                    &ctx.command,
                    "internal server error",
                )
                .await
                .map_err(|err| err.into())
            }
        };

        match result {
            Ok(_) => (),
            Err(reason) => {
                log::error!(
                    "failed to handle application command: (name: {}, reason: {:?})",
                    name,
                    reason
                );
                InteractionHelper::send_ephemeral(
                    &ctx.context.http,
                    &ctx.command,
                    "internal server error",
                )
                .await
                .unwrap();
            }
        }
    }
}
