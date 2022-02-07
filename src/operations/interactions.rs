use anyhow::Result;
use async_trait::async_trait;

use serenity::http::Http;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;

use std::collections::HashMap;

#[async_trait]
pub trait InteractionResponder {
    async fn send<D>(http: &Http, command: &ApplicationCommandInteraction, msg: D) -> Result<()>
    where
        D: ToString + Send;
}

#[async_trait]
pub trait InteractionDeferredResponder {
    async fn defer(http: &Http, command: &ApplicationCommandInteraction) -> Result<()>;
    async fn defer_respond<D>(http: &Http, command: &ApplicationCommandInteraction, msg: D) -> Result<()>
    where
        D: ToString + Send;
}

#[async_trait]
pub trait InteractionEphemeralResponder {
    async fn send_ephemeral<D>(
        http: &Http,
        command: &ApplicationCommandInteraction,
        msg: D,
    ) -> Result<()>
    where
        D: ToString + Send;
}

#[async_trait]
pub trait InteractionTableResponder {
    async fn send_table(
        http: &Http,
        command: &ApplicationCommandInteraction,
        table: HashMap<&str, String>,
    ) -> Result<()>;
}

pub trait InteractionArgumentExtractor {
    fn value_of_as_str<S: AsRef<str>>(
        command: &ApplicationCommandInteraction,
        key: S,
    ) -> Option<&str>;
}

pub struct InteractionHelper;

#[async_trait]
impl InteractionResponder for InteractionHelper {
    async fn send<D>(http: &Http, command: &ApplicationCommandInteraction, msg: D) -> Result<()>
    where
        D: ToString + Send,
    {
        command
            .create_interaction_response(http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.content(msg))
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InteractionDeferredResponder for InteractionHelper {
    async fn defer(http: &Http, command: &ApplicationCommandInteraction) -> Result<()> {
        command.defer(http).await?;
        Ok(())
    }

    async fn defer_respond<D>(http: &Http, command: &ApplicationCommandInteraction, msg: D) -> Result<()>
        where
            D: ToString + Send,
    {
        command
            .edit_original_interaction_response(http, |message| {
                message.content(msg)
            })
            .await;

        Ok(())
    }
}

#[async_trait]
impl InteractionEphemeralResponder for InteractionHelper {
    async fn send_ephemeral<D>(
        http: &Http,
        command: &ApplicationCommandInteraction,
        msg: D,
    ) -> Result<()>
    where
        D: ToString + Send,
    {
        command
            .create_interaction_response(http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.content(msg)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InteractionTableResponder for InteractionHelper {
    async fn send_table(
        http: &Http,
        command: &ApplicationCommandInteraction,
        table: HashMap<&str, String>,
    ) -> Result<()> {
        command
            .create_interaction_response(http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.create_embed(|embed| {
                            for (key, value) in table {
                                embed.field(key, value, false);
                            }
                            embed
                        })
                    })
            })
            .await?;
        Ok(())
    }
}

impl InteractionArgumentExtractor for InteractionHelper {
    fn value_of_as_str<S: AsRef<str>>(
        command: &ApplicationCommandInteraction,
        key: S,
    ) -> Option<&str> {
        for option in &command.data.options {
            if key.as_ref() == option.name {
                return option.value.as_ref().and_then(|v| v.as_str());
            }
        }
        None
    }
}
