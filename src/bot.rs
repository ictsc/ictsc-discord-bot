use crate::*;

use std::collections::HashMap;

use crate::commands::ask::AskCommand;
use crate::commands::join::JoinCommand;
use crate::commands::{ApplicationCommandContext, ReactionContext};
use crate::Result;
use serenity::async_trait;
use serenity::builder::*;
use serenity::http::Http;

use crate::commands::recreate::RecreateCommand;
use crate::SystemError::{NoSuchCategory, NoSuchRole};
use serenity::model::prelude::command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

type CommandCreator =
    Box<dyn FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand + Send>;
type CommandDefinitions<'a> = HashMap<&'a str, CommandCreator>;

static STAFF_CATEGORY_NAME: &str = "ICTSC2022 Staff";
static STAFF_ROLE_NAME: &str = "ICTSC2022 Staff";
static EVERYONE_ROLE_NAME: &str = "@everyone";
static ANNOUNCE_CHANNEL_NAME: &str = "announce";
static RANDOM_CHANNEL_NAME: &str = "random";
static TEXT_CHANNEL_NAME: &str = "text";
static VOICE_CHANNEL_NAME: &str = "voice";

fn get_readonly_permissions() -> Permissions {
    Permissions::ADD_REACTIONS
        | Permissions::READ_MESSAGE_HISTORY
}

fn get_team_permissions() -> Permissions {
    Permissions::ADD_REACTIONS
        | Permissions::ATTACH_FILES
        | Permissions::CHANGE_NICKNAME
        | Permissions::CONNECT
        | Permissions::EMBED_LINKS
        | Permissions::MENTION_EVERYONE
        | Permissions::READ_MESSAGE_HISTORY
        | Permissions::SEND_MESSAGES
        | Permissions::SEND_MESSAGES_IN_THREADS
        | Permissions::SEND_TTS_MESSAGES
        | Permissions::SPEAK
        | Permissions::USE_EXTERNAL_EMOJIS
        | Permissions::USE_VAD
        | Permissions::USE_SLASH_COMMANDS
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub token: String,
    pub guild_id: u64,
    pub application_id: u64,
    pub disabled_commands: Vec<String>,
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
    pub channel_name: String,
    pub role_name: String,
    pub invitation_code: String,
    pub user_group_id: String,
}

#[derive(Debug, Clone)]
pub struct ProblemConfiguration {
    pub id: String,
    pub name: String,
}

pub struct Bot {
    config: Configuration,
    ask_command: AskCommand<RoleManager, ThreadManager>,
    join_command: JoinCommand<RoleManager>,
    recreate_command: RecreateCommand<RoleManager, ProblemRecreateManager>,
}

impl Bot {
    pub fn new(config: Configuration) -> Self {
        let guild_id = GuildId(config.guild_id);

        let problem_recreate_manager = ProblemRecreateManager::new(
            config.recreate_service.baseurl.clone(),
            config.recreate_service.username.clone(),
            config.recreate_service.password.clone(),
        );

        let mut team_mapping = HashMap::new();
        team_mapping.insert(config.staff.password.clone(), String::from(STAFF_ROLE_NAME));
        config.teams.iter().for_each(|team| {
            team_mapping.insert(team.invitation_code.clone(), team.role_name.clone());
        });

        let ask_command = AskCommand::new(guild_id, RoleManager, ThreadManager);
        let join_command = JoinCommand::new(RoleManager, guild_id, team_mapping);
        let recreate_command = RecreateCommand::new(
            RoleManager,
            problem_recreate_manager,
            &config.teams,
            &config.problems,
        );

        Bot {
            config,
            ask_command,
            join_command,
            recreate_command,
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
                .description("チームに参加します。")
                .create_option(|option| {
                    option
                        .name("invitation_code")
                        .description("招待コード")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        }),
    );

    definitions
}

fn setup_application_command_definitions(
    disabled_commands: &[String],
) -> CommandDefinitions<'static> {
    let mut definitions = CommandDefinitions::new();

    if disabled_commands.contains(&"ping".to_string()) {
        tracing::info!("disabled discord command definition ping");
    } else {
        definitions.insert(
            "ping",
            Box::new(|command| command.name("ping").description("botの生存確認をします。")),
        );
    };

    if disabled_commands.contains(&"ask".to_string()) {
        tracing::info!("disabled discord command definition ask");
    } else {
        definitions.insert(
            "ask",
            Box::new(|command| {
                command
                    .name("ask")
                    .description("運営への質問スレッドを開始します")
                    .create_option(|option| {
                        option
                            .name("title")
                            .description("質問タイトル")
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            }),
        );
    };

    if disabled_commands.contains(&"recreate".to_string()) {
        tracing::info!("disabled discord command definition recreate");
    } else {
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
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            }),
        );
    };

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

        // TODO: review GatewayIntents
        // ref: https://docs.rs/serenity/latest/serenity/model/prelude/struct.GatewayIntents.html
        let mut client = Client::builder(token, GatewayIntents::all())
            .application_id(application_id)
            .event_handler(self)
            .await?;

        Ok(client.start().await?)
    }

    fn setup_client(&self) -> (GuildId, Http) {
        let token = &self.config.token;
        let application_id = self.config.application_id;

        (
            GuildId::from(self.config.guild_id),
            Http::new_with_application_id(token, application_id),
        )
    }

    #[tracing::instrument(skip_all)]
    pub async fn create_roles(&self) -> Result<()> {
        let (guild_id, ref http) = self.setup_client();

        tracing::info!("creating roles");

        let mut inputs = Vec::new();

        inputs.push(CreateRoleInput {
            name: String::from(STAFF_ROLE_NAME),
            color: 14942278,
            hoist: true,
            mentionable: true,
            permissions: Permissions::all(),
        });

        inputs.push(CreateRoleInput {
            name: String::from("@everyone"),
            permissions: Permissions::empty(),
            ..CreateRoleInput::default()
        });

        for team in &self.config.teams {
            inputs.push(CreateRoleInput {
                name: team.role_name.clone(),
                color: 0,
                hoist: true,
                mentionable: true,
                permissions: get_team_permissions(),
            })
        }

        RoleManager.sync(http, guild_id, inputs).await?;

        tracing::info!("create roles finished");

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_roles(&self) -> Result<()> {
        let (guild_id, ref http) = self.setup_client();

        tracing::info!("deleting all roles");

        RoleManager.delete_all(http, guild_id).await?;

        tracing::info!("delete all roles completed");

        Ok(())
    }

    fn create_topic(&self, team: &TeamConfiguration) -> String {
        format!("**__スコアサーバ__**

ユーザ登録URL：https://contest.ictsc.net/signup?invitation_code={invitation_code}&user_group_id={user_group_id}",
                invitation_code = team.invitation_code, user_group_id = team.user_group_id)
    }

    #[tracing::instrument(skip_all)]
    pub async fn create_channels(&self) -> Result<()> {
        let (guild_id, ref http) = self.setup_client();

        tracing::info!("fetching all roles");
        let roles: HashMap<_, _> = RoleManager
            .find_all(http, guild_id)
            .await?
            .into_iter()
            .map(|r| (r.name, r.id))
            .collect();

        let staff_role_id = *roles
            .get(STAFF_ROLE_NAME)
            .ok_or(NoSuchRole(STAFF_ROLE_NAME.into()))?;

        let everyone_role_id = *roles
            .get(EVERYONE_ROLE_NAME)
            .ok_or(NoSuchRole(EVERYONE_ROLE_NAME.into()))?;

        let default_permissions = vec![
            PermissionOverwrite {
                allow: Permissions::all(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(staff_role_id),
            },
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::all(),
                kind: PermissionOverwriteType::Role(everyone_role_id),
            },
        ];

        tracing::info!("creating categories");
        let mut inputs = Vec::new();

        inputs.push(CreateChannelInput {
            name: STAFF_CATEGORY_NAME.into(),
            kind: ChannelType::Category,
            permissions: default_permissions.clone(),
            ..CreateChannelInput::default()
        });

        for team in &self.config.teams {
            let team_role_id = *roles
                .get(&team.role_name)
                .ok_or(NoSuchRole(team.role_name.clone()))?;

            let mut permissions = default_permissions.clone();
            permissions.push(PermissionOverwrite {
                allow: get_team_permissions(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(team_role_id),
            });

            inputs.push(CreateChannelInput {
                name: team.channel_name.clone(),
                kind: ChannelType::Category,
                permissions,
                ..CreateChannelInput::default()
            });
        }

        let categories: HashMap<_, _> = ChannelManager
            .sync(http, guild_id, inputs)
            .await?
            .into_iter()
            .map(|c| (c.name, c.id))
            .collect();

        tracing::info!("creating channels");

        let mut inputs = Vec::new();

        let staff_category_id = *categories
            .get(STAFF_CATEGORY_NAME)
            .ok_or(NoSuchCategory(STAFF_CATEGORY_NAME.into()))?;

        let readonly_permissions = vec![
            PermissionOverwrite {
                allow: Permissions::all(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(staff_role_id),
            },
            PermissionOverwrite {
                allow: get_readonly_permissions(),
                deny: get_readonly_permissions().complement(),
                kind: PermissionOverwriteType::Role(everyone_role_id),
            },
        ];

        // everyone channels
        //inputs.push(CreateChannelInput {
        //    name: GUIDANCE_CHANNEL_NAME.into(),
        //    kind: ChannelType::Text,
        //    permissions: readonly_permissions.clone(),
        //    ..CreateChannelInput::default()
        //});

        inputs.push(CreateChannelInput {
            name: ANNOUNCE_CHANNEL_NAME.into(),
            kind: ChannelType::Text,
            permissions: readonly_permissions.clone(),
            ..CreateChannelInput::default()
        });

        inputs.push(CreateChannelInput {
            name: RANDOM_CHANNEL_NAME.into(),
            kind: ChannelType::Text,
            ..CreateChannelInput::default()
        });

        // staff channels
        inputs.push(CreateChannelInput {
            name: TEXT_CHANNEL_NAME.into(),
            kind: ChannelType::Text,
            category_id: Some(staff_category_id),
            permissions: default_permissions.clone(),
            ..CreateChannelInput::default()
        });

        inputs.push(CreateChannelInput {
            name: VOICE_CHANNEL_NAME.into(),
            kind: ChannelType::Voice,
            category_id: Some(staff_category_id),
            permissions: default_permissions.clone(),
            ..CreateChannelInput::default()
        });

        for team in &self.config.teams {
            let team_category_id = *categories
                .get(&team.channel_name)
                .ok_or(NoSuchCategory(team.channel_name.clone()))?;

            let team_role_id = *roles
                .get(&team.role_name)
                .ok_or(NoSuchRole(team.role_name.clone()))?;

            let mut permissions = default_permissions.clone();
            permissions.push(PermissionOverwrite {
                allow: get_team_permissions(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(team_role_id),
            });

            inputs.push(CreateChannelInput {
                name: TEXT_CHANNEL_NAME.into(),
                kind: ChannelType::Text,
                category_id: Some(team_category_id),
                topic: Some(self.create_topic(team)),
                permissions: permissions.clone(),
                ..CreateChannelInput::default()
            });

            inputs.push(CreateChannelInput {
                name: VOICE_CHANNEL_NAME.into(),
                kind: ChannelType::Voice,
                category_id: Some(team_category_id),
                permissions,
                ..CreateChannelInput::default()
            });
        }

        ChannelManager.sync(http, guild_id, inputs).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_channels(&self) -> Result<()> {
        let (guild_id, ref http) = self.setup_client();

        tracing::info!("deleting all channels");

        ChannelManager.delete_all(http, guild_id).await?;

        tracing::info!("delete all channels completed");

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_commands(&self) -> Result<()> {
        let (guild_id, ref http) = self.setup_client();

        tracing::info!("deleting all commands");

        for command in guild_id.get_application_commands(http).await? {
            tracing::debug!(?command, "deleting command");
            guild_id
                .delete_application_command(http, command.id)
                .await?;
        }

        tracing::info!("delete all commands completed");

        Ok(())
    }
}

impl Bot {
    async fn setup_global_application_command(&self, ctx: Context) {
        let definitions = setup_global_application_command_definitions();

        let commands = Command::get_global_application_commands(&ctx.http)
            .await
            .unwrap();
        for command in &commands {
            if !definitions.contains_key(command.name.as_str()) {
                tracing::debug!(?command, "delete global application command");
                Command::delete_global_application_command(&ctx.http, command.id)
                    .await
                    .unwrap();
            }
        }

        for (name, handler) in definitions {
            tracing::debug!(?name, "create global application command");
            Command::create_global_application_command(&ctx.http, handler)
                .await
                .unwrap();
        }
    }

    async fn setup_application_command(&self, ctx: Context, guild: Guild) {
        let definitions = setup_application_command_definitions(&self.config.disabled_commands);

        let commands = guild.get_application_commands(&ctx.http).await.unwrap();
        for command in &commands {
            if !definitions.contains_key(command.name.as_str()) {
                tracing::debug!(?command, "delete application command");
                guild
                    .delete_application_command(&ctx.http, command.id)
                    .await
                    .unwrap();
            }
        }

        for (name, handler) in definitions {
            tracing::debug!(?name, "create application command");
            guild
                .create_application_command(&ctx.http, handler)
                .await
                .unwrap();
        }
    }
}

impl Bot {
    #[tracing::instrument(skip_all)]
    async fn handle_command_ping(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        InteractionHelper::defer_respond(&ctx.context.http, &ctx.command, "pong!").await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_ask(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let title = InteractionHelper::value_of_as_str(&ctx.command, "title").unwrap();
        self.ask_command.run(ctx, title.into()).await
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_join(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let invitation_code =
            InteractionHelper::value_of_as_str(&ctx.command, "invitation_code").unwrap();
        self.join_command.run(ctx, invitation_code.into()).await
    }

    #[tracing::instrument(skip_all)]
    async fn handle_command_recreate(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let problem_code =
            InteractionHelper::value_of_as_str(&ctx.command, "problem_code").unwrap();
        self.recreate_command.run(ctx, problem_code.into()).await
    }

    #[tracing::instrument(skip_all, fields(
        guild_id = ?ctx.command.guild_id,
        channel_id = ?ctx.command.channel_id,
        user_id = ?ctx.command.user.id,
        user_name = ?ctx.command.user.name,
    ))]
    async fn handle_application_command(&self, ctx: ApplicationCommandContext) {
        let name = ctx.command.data.name.as_str();

        tracing::debug!("sending acknowledgement");
        let _ = InteractionHelper::defer(&ctx.context.http, &ctx.command).await;

        let result = if self.config.disabled_commands.contains(&name.to_string()) {
            tracing::info!(?name, "the command is disabled.");
            Err(errors::SystemError::UnhandledCommand(String::from(name)).into())
        } else {
            match name {
                "ping" => self.handle_command_ping(&ctx).await,
                "ask" => self.handle_command_ask(&ctx).await,
                "join" => self.handle_command_join(&ctx).await,
                "recreate" => self.handle_command_recreate(&ctx).await,
                _ => Err(errors::SystemError::UnhandledCommand(String::from(name)).into()),
            }
        };

        match result {
            Ok(_) => (),
            Err(err) => {
                tracing::error!(?err, "failed to handle application command");
                let _ = InteractionHelper::defer_respond(
                    &ctx.context.http,
                    &ctx.command,
                    format!("{} (id: {})", err, ctx.command.id),
                )
                .await;
            }
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
