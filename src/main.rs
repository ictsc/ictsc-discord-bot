mod config;
mod commands;

use config::*;

use anyhow::Result;
use clap::Parser;

use serenity::async_trait;

use serenity::model::interactions::application_command::ApplicationCommand;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Debug, Parser)]
#[clap(author, version)]
struct Arguments {
    #[clap(short = 'f', long = "filename")]
    config: String,
}

struct Bot {
    config: Configuration,
    guild: GuildId,
}

impl Bot {
    fn new(config: Configuration) -> Self {
        let guild = GuildId(config.discord.guild_id);

        Self {
            config,
            guild,
        }
    }

    async fn start(self) -> Result<()> {
        let token = &self.config.discord.token;
        let application_id = self.config.discord.application_id;

        let mut client = Client::builder(token)
            .application_id(application_id)
            .event_handler(self)
            .await?;

        Ok(client.start().await?)
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, _: Ready) {
        let commands = self.guild
            .set_application_commands(&ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        commands::ping::Command.create(command)
                    })
                    .create_application_command(|command| {
                        commands::join::Command.create(command)
                    })
            })
            .await;

        match commands {
            Ok(commands) => log::debug!("created application commands: {:?}", commands),
            Err(reason) => log::error!("failed to create application commands: {:?}", reason),
        };

        // let command = ApplicationCommand::create_global_application_command(&ctx.http, |command| {
        //         commands::join::Command.create(command)
        //     })
        //     .await;

        // match command {
        //     Ok(command) => log::debug!("created global application command: {:?}", command),
        //     Err(reason) => log::error!("failed to create global application command: {:?}", reason),
        // };

        log::info!("bot is ready");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            log::info!("command executed: {:?}", command);

            let result = match command.data.name.as_str() {
                "join" => commands::join::Command.run(ctx, command).await,
                "ping" => commands::ping::Command.run(ctx, command).await,
                _ => Ok(()),
            };

            match result {
                Ok(()) => log::info!("finished"),
                Err(err) => log::error!("error: {}", err),
            };
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Arguments = Arguments::parse();

    env_logger::init();

    let config = match Configuration::load(args.config) {
        Ok(config) => config,
        Err(err) => {
            log::error!("couldn't read config file: {:?}", err);
            return;
        }
    };

    let bot = Bot::new(config);

    if let Err(reason) = bot.start().await {
        log::error!("finished unsuccessfully: {:?}", reason);
        return;
    }
}
