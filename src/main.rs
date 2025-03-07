use anyhow::Result;
use bot::config::Configuration;
use bot::config::RedeployNotifiersConfiguration;
use bot::config::RedeployServiceConfiguration;
use bot::services::contestant::{ContestantService, FakeContestantService};
use bot::services::redeploy::regalia::{Regalia, RegaliaConfig};
use bot::services::redeploy::DiscordRedeployNotifier;
use bot::services::redeploy::FakeRedeployService;
use bot::services::redeploy::RState;
use bot::services::redeploy::RStateConfig;
use bot::services::redeploy::RedeployNotifier;
use bot::services::redeploy::RedeployService;
use bot::Bot;
use clap::Parser;
use clap::Subcommand;

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
    Sync,
    DeleteRoles,
    DeleteChannels,
    DeleteCommands,
}

fn build_redeploy_service(
    config: &Configuration,
) -> Result<Box<dyn RedeployService + Send + Sync>> {
    Ok(match &config.redeploy.service {
        RedeployServiceConfiguration::Rstate(rstate) => Box::new(RState::new(RStateConfig {
            baseurl: rstate.baseurl.clone(),
            username: rstate.username.clone(),
            password: rstate.password.clone(),
            problems: config.problems.clone(),
        })?),
        RedeployServiceConfiguration::Regalia(regalia) => Box::new(Regalia::new(RegaliaConfig {
            baseurl: regalia.baseurl.clone(),
            token: regalia.token.clone(),
        })?),
        RedeployServiceConfiguration::Fake => Box::new(FakeRedeployService),
    })
}

async fn build_redeploy_notifiers(
    config: &Configuration,
) -> Result<Vec<Box<dyn RedeployNotifier + Send + Sync>>> {
    let mut notifiers: Vec<Box<dyn RedeployNotifier + Send + Sync>> = Vec::new();
    for notifier_config in &config.redeploy.notifiers {
        match notifier_config {
            RedeployNotifiersConfiguration::Discord(discord) => {
                notifiers.push(Box::new(
                    DiscordRedeployNotifier::new(&config.discord.token, &discord.webhook_url)
                        .await?,
                ));
            },
        }
    }
    Ok(notifiers)
}

fn build_contestants_service(
    config: &Configuration,
) -> Result<Box<dyn ContestantService + Send + Sync>> {
    Ok(match &config.redeploy.service {
        RedeployServiceConfiguration::Regalia(regalia) => Box::new(Regalia::new(RegaliaConfig {
            baseurl: regalia.baseurl.clone(),
            token: regalia.token.clone(),
        })?),
        _ => Box::new(FakeContestantService),
    })
}

async fn sync(bot: &Bot) -> Result<()> {
    bot.sync_roles().await?;
    bot.sync_channels().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Arguments = Arguments::parse();
    let config = match Configuration::load(args.config) {
        Ok(config) => config,
        Err(err) => {
            tracing::error!(?err, "couldn't read config file");
            return;
        },
    };

    let contestants_service = match build_contestants_service(&config) {
        Ok(service) => service,
        Err(err) => {
            tracing::error!(?err, "couldn't instantiate contestants service");
            return;
        },
    };

    let redeploy_service = match build_redeploy_service(&config) {
        Ok(service) => service,
        Err(err) => {
            tracing::error!(?err, "couldn't instantiate redeploy service");
            return;
        },
    };

    let redeploy_notifiers = match build_redeploy_notifiers(&config).await {
        Ok(notifiers) => notifiers,
        Err(err) => {
            tracing::error!("couldn't instantiate DiscordRedeployNotifier: {:?}", err);
            return;
        },
    };

    let bot = Bot::new(
        config.discord.token,
        config.discord.application_id,
        config.discord.guild_id,
        config.staff.password,
        config.teams,
        config.problems,
        redeploy_service,
        redeploy_notifiers,
        config.discord.configure_channel_topics,
    );

    let result = match args.command {
        Commands::Start => bot.start().await,
        Commands::Sync => sync(&bot).await,
        Commands::DeleteRoles => bot.delete_roles().await,
        Commands::DeleteChannels => bot.delete_channels().await,
        Commands::DeleteCommands => bot.delete_commands().await,
    };

    if let Err(reason) = result {
        tracing::error!("finished unsuccessfully: {:?}", reason);
    }
}
