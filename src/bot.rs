use crate::*;

use std::collections::HashMap;

use crate::commands::ask::AskCommand;
use crate::commands::join::JoinCommand;
use crate::commands::whoami::WhoAmICommand;
use crate::commands::{ApplicationCommandContext, ReactionContext};
use crate::Result;
use serenity::async_trait;
use serenity::builder::*;
use serenity::http::Http;

use crate::commands::recreate::RecreateCommand;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

type CommandCreator =
    Box<dyn FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand + Send>;
type CommandDefinitions<'a> = HashMap<&'a str, CommandCreator>;

static STAFF_ROLE_NAME: &'static str = "ICTSC2021 Staff";
static TEAM_TEXT_CHANNEL_NAME: &'static str = "text";
static TEAM_VOICE_CHANNEL_NAME: &'static str = "voice";

#[derive(Debug, Clone)]
pub struct Configuration {
    pub token: String,
    pub guild_id: u64,
    pub application_id: u64,
    pub staff: StaffConfiguration,
    pub recreate_service: RecreateServiceConfiguration,
    pub teams: Vec<TeamConfiguration>,
    pub problems: Vec<ProblemConfiguration>,
}

#[derive(Debug, Clone)]
pub struct StaffConfiguration {
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct RecreateServiceConfiguration {
    pub baseurl: String,
    pub username: String,
    pub password: String,
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
    pub name: String,
}

pub struct Bot {
    config: Configuration,
    ask_command: AskCommand<UserManager, ThreadManager>,
    join_command: JoinCommand<RoleManager>,
    recreate_command: RecreateCommand<RoleManager, ProblemRecreateManager>,
    whoami_command: WhoAmICommand<UserManager>,
}

impl Bot {
    pub fn new(config: Configuration) -> Self {
        let guild_id = GuildId(config.guild_id);

        let problemRecreateManager = ProblemRecreateManager::new(
            config.recreate_service.baseurl.clone(),
            config.recreate_service.username.clone(),
            config.recreate_service.password.clone(),
        );

        let mut team_mapping  = HashMap::new();
        team_mapping.insert(config.staff.password.clone(), String::from(STAFF_ROLE_NAME));
        config.teams.iter()
            .for_each(|team| {
                team_mapping.insert(team.invitation_code.clone(), team.role_name.clone());
            });

        let ask_command = AskCommand::new(UserManager, ThreadManager);
        let join_command = JoinCommand::new(RoleManager, guild_id, team_mapping);
        let recreate_command = RecreateCommand::new(RoleManager, problemRecreateManager, &config.teams, &config.problems);
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
    #[tracing::instrument(skip_all, fields(
        guild_id = ?guild.id
    ))]
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        tracing::debug!("guild created");

        self.setup_application_command(ctx, guild).await;
    }

    #[tracing::instrument(skip_all)]
    async fn ready(&self, ctx: Context, _: Ready) {
        tracing::info!("bot is ready");

        self.setup_global_application_command(ctx).await;
    }

    #[tracing::instrument(skip_all, fields(
        interaction_kind = ?interaction.kind(),
        interaction_id = ?interaction.id(),
    ))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        tracing::debug!("called interaction_create");

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

    #[tracing::instrument(skip_all, fields(
        guild_id = ?reaction.guild_id,
        message_id = ?reaction.message_id,
        user_id = ?reaction.user_id,
        channel_id = ?reaction.channel_id,
    ))]
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        tracing::debug!("called reaction_add");

        self.handle_reaction(ReactionContext {
            context: ctx,
            reaction,
        })
        .await;
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

    pub async fn create_roles(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        let mut roles = Vec::new();

        roles.push(CreateRoleInput {
            name: String::from(STAFF_ROLE_NAME),
            color: 14942278,
            hoist: true,
            mentionable: true,
            permissions: Permissions::all(),
        });

        for team in &self.config.teams {
            roles.push(CreateRoleInput {
                name: team.role_name.clone(),
                color: 0,
                hoist: true,
                mentionable: true,
                permissions: Permissions::empty(),
            })
        }

        RoleManager.sync_bulk(http, guild_id, roles).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_roles(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        tracing::info!("deleting all roles");

        RoleManager.delete_all(http, guild_id).await;

        tracing::info!("delete all roles completed");

        Ok(())
    }

    pub async fn create_channels(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        let mut categories = Vec::new();

        categories.push(CreateChannelInput {
            name: String::from("admin"),
            kind: ChannelKind::Category,
            ..CreateChannelInput::default()
        });

        for team in &self.config.teams {
            categories.push(CreateChannelInput {
                name: team.channel_name.clone(),
                kind: ChannelKind::Category,
                ..CreateChannelInput::default()
            });
        }

        let categories = ChannelManager.sync(http, guild_id, categories).await?;
        let mut categories_table = HashMap::new();
        for category in categories {
            categories_table.insert(category.name, category.id);
        }

        println!("{:?}", categories_table);

        let mut channels = Vec::new();

        let category_id = categories_table.get("admin")
            .expect("channel name is invalid").clone();

        channels.push(CreateChannelInput {
            name: String::from("admin"),
            kind: ChannelKind::Text,
            category_id: Some(category_id),
            ..CreateChannelInput::default()
        });

        for team in &self.config.teams {
            let category_id = categories_table.get(&team.channel_name)
                .expect("channel name is invalid").clone();

            channels.push(CreateChannelInput {
                name: String::from(TEAM_TEXT_CHANNEL_NAME),
                kind: ChannelKind::Text,
                category_id: Some(category_id),
                ..CreateChannelInput::default()
            });

            channels.push(CreateChannelInput {
                name: String::from(TEAM_VOICE_CHANNEL_NAME),
                kind: ChannelKind::Voice,
                category_id: Some(category_id),
                ..CreateChannelInput::default()
            });
        }

        ChannelManager.sync(http, guild_id, channels).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_channels(&self) -> Result<()> {
        let token = &self.config.token;
        let guild_id = GuildId::from(self.config.guild_id);
        let application_id = self.config.application_id;

        let http = &Http::new_with_token_application_id(token, application_id);

        tracing::info!("deleting all channels");

        ChannelManager.delete_all(http, guild_id).await;

        tracing::info!("delete all channels completed");

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
                tracing::debug!("delete global application command: {:?}", command);
                ApplicationCommand::delete_global_application_command(&ctx.http, command.id)
                    .await
                    .unwrap();
            }
        }

        for (name, handler) in definitions {
            tracing::debug!("create global application command: {:?}", name);
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
                tracing::debug!("delete application command: {:?}", command);
                guild
                    .delete_application_command(&ctx.http, command.id)
                    .await
                    .unwrap();
            }
        }

        for (name, handler) in definitions {
            tracing::debug!("create application command: {:?}", name);
            guild
                .create_application_command(&ctx.http, handler)
                .await
                .unwrap();
        }
    }

    async fn teardown_application_command(&self, ctx: Context, guild: Guild) {
        let commands = guild.get_application_commands(&ctx.http).await.unwrap();

        for command in &commands {
            tracing::debug!("delete application command: {:?}", command);
            guild
                .delete_application_command(&ctx.http, command.id)
                .await
                .unwrap();
        }
    }
}

impl Bot {
    #[tracing::instrument(skip_all)]
    async fn handle_command_ping(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        InteractionHelper::send(&ctx.context.http, &ctx.command, "pong!").await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_ask(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let summary = InteractionHelper::value_of_as_str(&ctx.command, "summary").unwrap();
        Ok(self.ask_command.run(ctx, summary.into()).await?)
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_whoami(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        Ok(self.whoami_command.run(ctx).await?)
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_join(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let invitation_code =
            InteractionHelper::value_of_as_str(&ctx.command, "invitation_code").unwrap();
        Ok(self.join_command.run(ctx, invitation_code.into()).await?)
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_recreate(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let problem_code =
            InteractionHelper::value_of_as_str(&ctx.command, "problem_code").unwrap();
        Ok(self.recreate_command.run(ctx, problem_code.into()).await?)
    }

    #[tracing::instrument(skip_all, fields(
        guild_id = ?ctx.command.guild_id,
        channel_id = ?ctx.command.channel_id,
        user_id = ?ctx.command.user.id,
        user_name = ?ctx.command.user.name,
    ))]
    async fn handle_application_command(&self, ctx: ApplicationCommandContext) {
        let name = ctx.command.data.name.as_str();

        let result = match name {
            "ping" => self.handle_command_ping(&ctx).await,
            "whoami" => self.handle_command_whoami(&ctx).await,
            "ask" => self.handle_command_ask(&ctx).await,
            "join" => self.handle_command_join(&ctx).await,
            "recreate" => self.handle_command_recreate(&ctx).await,
            _ => Err(errors::SystemError::UnhandledCommand(String::from(name)).into()),
        };

        match result {
            Ok(_) => (),
            Err(err) => {
                tracing::error!(?err, "failed to handle application command");
                let _ = InteractionHelper::send_ephemeral(
                    &ctx.context.http,
                    &ctx.command,
                    format!("{} (interaction_id: {})", err, ctx.command.id),
                ).await;
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn handle_reaction(&self, ctx: ReactionContext) {
        let result = self.recreate_command.add_reaction(&ctx).await;

        if let Err(err) = result {
            tracing::error!("failed to handle reaction: {:?}", err);
        }
    }
}
