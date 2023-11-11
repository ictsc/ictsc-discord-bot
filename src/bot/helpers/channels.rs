use serenity::model::prelude::*;

use super::HelperResult;
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
    pub async fn create_channel(&self, definition: &GuildChannelDefinition) -> HelperResult<()> {
        let definition = definition.clone();
        self.guild_id
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
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_channels<T: AsRef<[ChannelType]>>(
        &self,
        kinds: T,
    ) -> HelperResult<Vec<GuildChannel>> {
        Ok(self
            .guild_id
            .channels(&self.discord_client)
            .await?
            .into_values()
            .filter(|channel| kinds.as_ref().contains(&channel.kind))
            .collect())
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_categories(&self) -> HelperResult<Vec<GuildChannel>> {
        Ok(self
            .guild_id
            .channels(&self.discord_client)
            .await?
            .into_values()
            .filter(|channel| channel.kind == ChannelType::Category)
            .collect())
    }

    #[tracing::instrument(skip_all, fields(
        category = ?category,
        definition = ?definition,
    ))]
    pub async fn edit_channel(
        &self,
        category: &mut GuildChannel,
        definition: &GuildChannelDefinition,
    ) -> HelperResult<()> {
        if category.kind != definition.kind {
            anyhow::anyhow!("failed to edit category: kind is not matched");
        }

        category
            .edit(&self.discord_client, |edit| {
                edit.name(&definition.name)
                    .category(definition.category)
                    .permissions(definition.permissions.clone())
            })
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(category = ?category))]
    pub async fn delete_channel(&self, category: &mut GuildChannel) -> HelperResult<()> {
        category.delete(&self.discord_client).await?;
        Ok(())
    }
}
