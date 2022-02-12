use crate::Result;
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
    async fn create(
        &self,
        http: &Http,
        guild_id: GuildId,
        input: CreateTextChannelInput,
    ) -> Result<GuildChannel>;
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
    async fn find_all(
        &self,
        http: &Http,
        guild_id: GuildId,
    ) -> Result<Vec<GuildChannel>>;
}

#[async_trait]
pub trait TextChannelDeleter {
    async fn delete(&self, http: &Http, guild_id: GuildId, channel_id: ChannelId) -> Result<()>;
}

#[async_trait]
pub trait TextChannelSyncer: TextChannelCreator + TextChannelFinder + TextChannelDeleter {
    async fn sync_cached(
        &self,
        http: &Http,
        guild_id: GuildId,
        channels: &[GuildChannel],
        input: CreateTextChannelInput,
    ) -> Result<GuildChannel> {
        let channels: Vec<_> = channels.iter()
            .filter(|channel| channel.name == input.name)
            .collect();

        match channels.len() {
            1 => {
                // TODO: handling parameter change
                return Ok(channels[0].clone());
            }
            _ => {
                for channel in channels {
                    self.delete(http, guild_id, channel.id).await?;
                }
                return self.create(http, guild_id, input).await;
            }
        };
    }

    async fn sync(
        &self,
        http: &Http,
        guild_id: GuildId,
        input: CreateTextChannelInput,
    ) -> Result<GuildChannel> {
        let channels = self.find_all(http, guild_id).await?;

        self.sync_cached(http, guild_id, &channels, input).await
    }

    async fn sync_bulk(
        &self,
        http: &Http,
        guild_id: GuildId,
        inputs: Vec<CreateTextChannelInput>,
    ) -> Result<Vec<GuildChannel>> {
        let channels = self.find_all(http, guild_id).await?;

        let mut results = Vec::new();

        for input in inputs {
            results.push(self.sync_cached(http, guild_id, &channels, input).await?);
        }

        Ok(results)
    }
}

pub struct TextChannelManager;

#[async_trait]
impl TextChannelCreator for TextChannelManager {
    async fn create(
        &self,
        http: &Http,
        guild_id: GuildId,
        input: CreateTextChannelInput,
    ) -> Result<GuildChannel> {
        Ok(guild_id
            .create_channel(http, |channel| {
                channel.name(input.name).kind(ChannelType::Text);

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

    async fn find_all(&self, http: &Http, guild_id: GuildId) -> Result<Vec<GuildChannel>> {
        Ok(guild_id.channels(http)
            .await?
            .into_iter()
            .map(|(_, channel)| channel)
            .filter(|channel| channel.kind == ChannelType::Text)
            .collect())
    }
}

#[async_trait]
impl TextChannelDeleter for TextChannelManager {
    async fn delete(&self, http: &Http, _guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
        channel_id.delete(http).await?;
        Ok(())
    }
}

impl TextChannelSyncer for TextChannelManager {}
