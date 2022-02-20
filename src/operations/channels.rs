use crate::Result;
use async_trait::async_trait;
use serenity::http::Http;
use serenity::model::prelude::*;

#[derive(Debug)]
pub struct CreateChannelInput {
    pub name: String,
    pub kind: ChannelType,
    pub category_id: Option<ChannelId>,
    pub topic: Option<String>,
    pub permissions: Vec<PermissionOverwrite>,
}

impl Default for CreateChannelInput {
    fn default() -> Self {
        Self {
            name: String::default(),
            kind: ChannelType::Unknown,
            category_id: Option::default(),
            topic: Option::default(),
            permissions: Vec::default(),
        }
    }
}

pub struct ChannelManager;

#[async_trait]
pub trait ChannelCreator {
    async fn create(
        &self,
        http: &Http,
        guild_id: GuildId,
        input: CreateChannelInput,
    ) -> Result<GuildChannel>;
}

#[async_trait]
pub trait ChannelFinder {
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
    async fn find_all(&self, http: &Http, guild_id: GuildId) -> Result<Vec<GuildChannel>>;
}

#[async_trait]
pub trait ChannelDeleter {
    async fn delete(&self, http: &Http, guild_id: GuildId, channel_id: ChannelId) -> Result<()>;
    async fn delete_all(&self, http: &Http, guild_id: GuildId) -> Result<()>;
}

#[async_trait]
pub trait ChannelSyncer {
    async fn sync(
        &self,
        http: &Http,
        guild_id: GuildId,
        inputs: Vec<CreateChannelInput>,
    ) -> Result<Vec<GuildChannel>>;
}

#[async_trait]
impl ChannelCreator for ChannelManager {
    async fn create(
        &self,
        http: &Http,
        guild_id: GuildId,
        input: CreateChannelInput,
    ) -> Result<GuildChannel> {
        Ok(guild_id
            .create_channel(http, |channel| {
                channel.name(input.name).kind(input.kind.into());
                channel.permissions(input.permissions);

                match input.category_id {
                    Some(id) => channel.category(id),
                    None => channel,
                }
            })
            .await?)
    }
}

#[async_trait]
impl ChannelFinder for ChannelManager {
    async fn find_by_id(
        &self,
        http: &Http,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<Option<GuildChannel>> {
        for (id, channel) in guild_id.channels(http).await? {
            if id == channel_id {
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
            if name.as_ref() == channel.name {
                result.push(channel);
            }
        }
        Ok(result)
    }

    async fn find_all(&self, http: &Http, guild_id: GuildId) -> Result<Vec<GuildChannel>> {
        Ok(guild_id
            .channels(http)
            .await?
            .into_iter()
            .map(|(_, channel)| channel)
            .collect())
    }
}

#[async_trait]
impl ChannelDeleter for ChannelManager {
    async fn delete(&self, http: &Http, _guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
        channel_id.delete(http).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, http))]
    async fn delete_all(&self, http: &Http, guild_id: GuildId) -> Result<()> {
        for (_, channel) in &guild_id.channels(http).await? {
            tracing::debug!(channel_id = ?channel.id, channel_name = ?channel.name, "deleting channel");
            channel.delete(http).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T> ChannelSyncer for T
where
    T: ChannelCreator + ChannelDeleter + ChannelFinder + Sync,
{
    #[tracing::instrument(skip_all)]
    async fn sync(
        &self,
        http: &Http,
        guild_id: GuildId,
        inputs: Vec<CreateChannelInput>,
    ) -> Result<Vec<GuildChannel>> {
        let channels = self.find_all(http, guild_id).await?;

        let mut results = Vec::new();

        for input in inputs {
            tracing::debug!(?input, "syncing channel");

            let filtered: Vec<_> = channels
                .iter()
                .filter(|channel| {
                    channel.name == input.name && channel.kind == input.kind.into()
                        && channel.category_id == input.category_id
                })
                .collect();

            match filtered.len() {
                1 => {
                    let mut channel = filtered[0].clone();

                    tracing::debug!(?channel, ?input, "channel found, syncing");

                    channel.edit(http, |channel| {
                        if let Some(topic) = input.topic {
                            channel.topic(topic);
                        }
                        channel.permissions(input.permissions)
                    }).await?;

                    results.push(channel);
                }
                _ => {
                    tracing::debug!(role_name = ?input.name, "channel not found or several channels found, updating");
                    for channel in filtered {
                        self.delete(http, guild_id, channel.id).await?;
                    }
                    results.push(self.create(http, guild_id, input).await?);
                }
            }
        }

        Ok(results)
    }
}
