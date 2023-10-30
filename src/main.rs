use bot::config::Configuration;
use bot::services::redeploy::FakeRedeployService;
use bot::Bot;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version)]
struct Arguments {
    #[clap(short = 'f', long = "filename")]
    config: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Start,
    SyncRoles,
    DeleteRoles,
    SyncChannels,
    DeleteChannels,
    DeleteCommands,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Arguments = Arguments::parse();
    let config = match Configuration::load(args.config) {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("couldn't read config file: {:?}", err);
            return;
        }
    };

    let redeploy_service = FakeRedeployService;

    let bot = Bot::new(
        config.discord.token,
        config.discord.application_id,
        config.discord.guild_id,
        config.staff.password,
        config.teams,
        config.problems,
        Box::new(redeploy_service),
    );

    let result = match args.command {
        Commands::Start => bot.start().await,
        Commands::SyncRoles => bot.sync_roles().await,
        Commands::DeleteRoles => bot.delete_roles().await,
        Commands::SyncChannels => bot.sync_channels().await,
        Commands::DeleteChannels => bot.delete_channels().await,
        Commands::DeleteCommands => bot.delete_commands().await,
    };

    if let Err(reason) = result {
        tracing::error!("finished unsuccessfully: {:?}", reason);
    }
}
