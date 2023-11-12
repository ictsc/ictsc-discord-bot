use serenity::model::prelude::*;

use super::HelperResult;
use crate::bot::helpers::HelperError;
use crate::bot::Bot;

#[derive(Clone, Debug, derive_builder::Builder)]
pub struct GuildChannelDefinition {
    pub name: String,
    pub kind: ChannelType,
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
        Ok(self
            .guild_id
            .create_channel(&self.discord_client, |channel| {
                channel
                    .name(definition.name)
                    .kind(definition.kind)
                    .permissions(definition.permissions);

                match definition.category {
                    Some(category_id) => channel.category(category_id),
                    None => channel,
                }
            })
            .await?)
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
        Ok(channel
            .edit(&self.discord_client, |edit| {
                edit.name(&definition.name)
                    .category(definition.category)
                    .permissions(definition.permissions.clone())
            })
            .await?)
    }

    #[tracing::instrument(skip_all, fields(channel = ?channel))]
    pub async fn delete_channel(&self, channel: &mut GuildChannel) -> HelperResult<Channel> {
        tracing::trace!("Delete channel");
        Ok(channel.delete(&self.discord_client).await?)
    }
}
