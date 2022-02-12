mod config;

use config::*;

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
    CreateRoles,
    CreateChannels,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Arguments = Arguments::parse();
    let config = match Configuration::load(args.config) {
        Ok(config) => config,
        Err(err) => {
            log::error!("couldn't read config file: {:?}", err);
            return;
        }
    };

    let bot = bot::Bot::new(config.into());

    let result = match args.command {
        Commands::Start => bot.start().await,
        Commands::CreateRoles => bot.create_roles().await,
        Commands::CreateChannels => bot.create_channels().await,
    };

    if let Err(reason) = result {
        log::error!("finished unsuccessfully: {:?}", reason);
    }
}
