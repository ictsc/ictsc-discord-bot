use serenity::all::CreateChannel;
use serenity::all::CreateThread;
use serenity::all::EditChannel;
use serenity::all::EditThread;
use serenity::model::prelude::*;

use super::HelperResult;
use crate::bot::helpers::HelperError;
use crate::bot::Bot;

#[derive(Clone, Debug, derive_builder::Builder)]
pub struct GuildChannelDefinition {
    pub name: String,
    pub kind: ChannelType,
    #[builder(default)]
    pub topic: Option<String>,
    #[builder(default)]
    pub category: Option<ChannelId>,
    #[builder(default)]
    pub permissions: Vec<PermissionOverwrite>,
}

// Guildのチャンネルを操作するためのヘルパー関数
impl Bot {
    #[tracing::instrument(skip_all, fields(definition = ?definition))]
    pub async fn create_channel(
        &self,
        definition: &GuildChannelDefinition,
    ) -> HelperResult<GuildChannel> {
        tracing::trace!("Create channel");
        let definition = definition.clone();

        let mut create_channel = CreateChannel::new(definition.name)
            .kind(definition.kind)
            .permissions(definition.permissions);

        create_channel = match definition.topic {
            Some(topic) => create_channel.topic(topic),
            None => create_channel,
        };

        create_channel = match definition.category {
            Some(category_id) => create_channel.category(category_id),
            None => create_channel,
        };

        Ok(self
            .guild_id
            .create_channel(&self.discord_client, create_channel)
            .await?)
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_channel(&self, id: ChannelId) -> HelperResult<Channel> {
        tracing::trace!("Get channel");
        Ok(self.discord_client.get_channel(id).await?)
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_channels<T: AsRef<[ChannelType]>>(
        &self,
        kinds: T,
    ) -> HelperResult<Vec<GuildChannel>> {
        tracing::trace!("Get channels");
        Ok(self
            .guild_id
            .channels(&self.discord_client)
            .await?
            .into_values()
            .filter(|channel| kinds.as_ref().contains(&channel.kind))
            .collect())
    }

    #[tracing::instrument(skip_all, fields(
        channel = ?channel,
        definition = ?definition,
    ))]
    pub async fn edit_channel(
        &self,
        channel: &mut GuildChannel,
        definition: &GuildChannelDefinition,
    ) -> HelperResult<()> {
        tracing::trace!("Edit channel");
        if channel.kind != definition.kind {
            return Err(HelperError::InvalidChannelKindError);
        }

        let mut edit_channel = EditChannel::new()
            .name(&definition.name)
            .category(definition.category)
            .permissions(definition.permissions.clone());

        edit_channel = match &definition.topic {
            Some(topic) => edit_channel.topic(topic),
            None => edit_channel,
        };

        Ok(channel.edit(&self.discord_client, edit_channel).await?)
    }

    #[tracing::instrument(skip_all, fields(channel = ?channel))]
    pub async fn archive_thread(&self, channel: &mut GuildChannel) -> HelperResult<()> {
        tracing::trace!("Edit channel");
        if channel.kind != ChannelType::PublicThread && channel.kind != ChannelType::PrivateThread {
            return Err(HelperError::InvalidChannelKindError);
        }
        Ok(channel
            .edit_thread(&self.discord_client, EditThread::new().archived(true))
            .await?)
    }

    #[tracing::instrument(skip_all, fields(channel = ?channel))]
    pub async fn delete_channel(&self, channel: &mut GuildChannel) -> HelperResult<GuildChannel> {
        tracing::trace!("Delete channel");
        Ok(channel.delete(&self.discord_client).await?)
    }

    #[tracing::instrument(skip_all, fields(channel = ?channel))]
    pub async fn create_public_thread(
        &self,
        channel: &GuildChannel,
        message: &Message,
        title: &str,
    ) -> HelperResult<GuildChannel> {
        tracing::trace!("Create public thread");
        Ok(channel
            .create_thread_from_message(&self.discord_client, message, CreateThread::new(title))
            .await?)
    }
}
