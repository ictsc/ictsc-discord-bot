use anyhow::Result;
use async_trait::async_trait;
use serenity::http::Http;
use serenity::model::prelude::*;

#[derive(Default)]
pub struct CreateTextChannelInput {
    pub name: String,
    pub category_id: Option<ChannelId>,
}

#[async_trait]
pub trait TextChannelCreator {
    async fn create(&self, http: &Http, guild_id: GuildId, input: CreateTextChannelInput) -> Result<GuildChannel>;
}

#[async_trait]
pub trait TextChannelFinder {
    async fn find_by_id(
        &self,
        http: &Http,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<Option<GuildChannel>>;
    async fn find_by_name<S: AsRef<str> + Send>(
        &self,
        http: &Http,
        guild_id: GuildId,
        name: S,
    ) -> Result<Vec<GuildChannel>>;
}

#[async_trait]
pub trait TextChannelDeleter {
    async fn delete(&self, http: &Http, guild_id: GuildId, channel_id: ChannelId) -> Result<()>;
}

#[async_trait]
pub trait TextChannelSyncer: TextChannelCreator + TextChannelFinder + TextChannelDeleter {
    async fn sync(&self, http: &Http, guild_id: GuildId, input: CreateTextChannelInput) -> Result<()> {
        let mut channels = self.find_by_name(http, guild_id, &input.name).await?;

        match channels.len() {
            1 => {
                channels[0]
                    .edit(http, |channel| {
                        channel.category(input.category_id)
                    })
                    .await?;
            }
            _ => {
                for channel in channels {
                    self.delete(http, guild_id, channel.id).await?;
                }
                self.create(http, guild_id, input).await?;
            }
        };

        Ok(())
    }
}

pub struct TextChannelManager;

#[async_trait]
impl TextChannelCreator for TextChannelManager {
    async fn create(&self, http: &Http, guild_id: GuildId, input: CreateTextChannelInput) -> Result<GuildChannel> {
        Ok(guild_id
            .create_channel(http, |channel| {
                channel.name(input.name)
                    .kind(ChannelType::Text);

                match input.category_id {
                    Some(id) => channel.category(id),
                    None => channel,
                }
            })
            .await?)
    }
}

#[async_trait]
impl TextChannelFinder for TextChannelManager {
    async fn find_by_id(
        &self,
        http: &Http,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<Option<GuildChannel>> {
        for (id, channel) in guild_id.channels(http).await? {
            if channel.kind == ChannelType::Text && id == channel_id {
                return Ok(Some(channel));
            }
        }
        Ok(None)
    }

    async fn find_by_name<S: AsRef<str> + Send>(
        &self,
        http: &Http,
        guild_id: GuildId,
        name: S,
    ) -> Result<Vec<GuildChannel>> {
        let mut result: Vec<_> = vec![];
        for (_, channel) in guild_id.channels(http).await? {
            if channel.kind == ChannelType::Text && name.as_ref() == channel.name {
                result.push(channel);
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl TextChannelDeleter for TextChannelManager {
    async fn delete(&self, http: &Http, guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
        channel_id.delete(http).await?;
        Ok(())
    }
}

impl TextChannelSyncer for TextChannelManager {}
