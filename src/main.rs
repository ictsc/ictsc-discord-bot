mod config;

use config::*;


use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version)]
struct Arguments {
    #[clap(short = 'f', long = "filename")]
    config: String,
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
    if let Err(reason) = bot.start().await {
        log::error!("finished unsuccessfully: {:?}", reason);
        
    }
}
