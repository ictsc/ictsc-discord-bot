use std::collections::HashMap;

use crate::CommandResult;
use serenity::model::prelude::*;

use super::Bot;

static STAFF_CATEGORY_NAME: &str = "ICTSC2023 Staff";

#[derive(Clone, Debug, derive_builder::Builder)]
struct GuildChannelDefinition {
    name: String,
    kind: ChannelType,
    permissions: Vec<PermissionOverwrite>,
}

impl Bot {
    #[tracing::instrument(skip_all)]
    pub async fn sync_channels(&self) -> CommandResult<()> {
        tracing::info!("sync categories");

        let mut categories = Vec::new();

        categories.push(
            GuildChannelDefinitionBuilder::default()
                .name(STAFF_CATEGORY_NAME.to_string())
                .permissions(Vec::new())
                .build()?,
        );

        for team in &self.teams {
            categories.push(
                GuildChannelDefinitionBuilder::default()
                    .name(team.category_name.clone())
                    .permissions(Vec::new())
                    .build()?,
            );
        }

        self._sync_channels(&[ChannelType::Category], categories).await?;

        tracing::info!("sync channels");

        let _: HashMap<_, _> = self.get_channels(&[ChannelType::Category]).await?
            .into_iter()
            .map(|category| (category.name.clone(), category))
            .collect();

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_channels(&self) -> CommandResult<()> {
        tracing::info!("delete all channels");
        for (channel_id, channel) in self.guild_id.channels(&self.discord_client).await? {
            tracing::debug!(?channel, "delete channel");
            channel_id.delete(&self.discord_client).await?;
        }
        Ok(())
    }
}

impl Bot {
    async fn _sync_channels<K, T>(
        &self,
        kinds: K,
        definitions: T,
    ) -> CommandResult<()> 
    where
        K: AsRef<[ChannelType]>,
        T: AsRef<[GuildChannelDefinition]>,
    {
        tracing::debug!("fetch current channels");
        let mut channels = self.get_channels(kinds).await?;

        tracing::debug!("sync defined channels");
        for definition in definitions.as_ref() {
            tracing::debug!(?definition, "sync channels");

            let matched_channels: Vec<_> = channels
                .iter_mut()
                .filter(|c| c.name == definition.name)
                .collect();

            if matched_channels.len() == 1 {
                let channel = matched_channels.into_iter().nth(0).unwrap();
                if self.check_channel_synced(channel, definition) {
                    tracing::debug!(
                        channel_id = ?channel.id,
                        channel_name = ?channel.name,
                        "target channel is created and synced, skip"
                    );
                    continue;
                }
                tracing::debug!(
                    ?channel,
                    ?definition,
                    "channel is created but not synced, update channel"
                );
                self.edit_channel(channel, &definition).await?;
                continue;
            }

            if matched_channels.len() != 0 {
                tracing::debug!(
                    ?matched_channels,
                    "several matched channels are found, delete them"
                );
                for channel in matched_channels {
                    self.delete_channel(channel).await?;
                }
            }

            tracing::debug!(?definition, "create channel");
            self.create_channel(&definition).await?;
        }

        tracing::debug!("delete not-defined channels");
        for channel in channels.iter_mut() {
            let found = definitions
                .as_ref()
                .iter()
                .find(|d| d.name == channel.name)
                .is_some();

            if !found {
                tracing::debug!(?channel, "delete category");
                self.delete_channel(channel).await?;
            }
        }

        Ok(())
    }

    fn check_channel_synced(
        &self,
        category: &GuildChannel,
        definition: &GuildChannelDefinition,
    ) -> bool {
        category.kind == ChannelType::Category
            && category.name == definition.name
            && category.permission_overwrites == definition.permissions
    }
}

// CRUD operation for category
impl Bot {
    #[tracing::instrument(skip_all, fields(definition = ?definition))]
    async fn create_channel(&self, definition: &GuildChannelDefinition) -> CommandResult<()> {
        let definition = definition.clone();
        self.guild_id
            .create_channel(&self.discord_client, |channel| {
                channel
                    .name(definition.name)
                    .kind(definition.kind)
                    .permissions(definition.permissions)
            })
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn get_channels<T: AsRef<[ChannelType]>>(&self, kinds: T) -> CommandResult<Vec<GuildChannel>> {
        Ok(self
            .guild_id
            .channels(&self.discord_client)
            .await?
            .into_values()
            .filter(|channel| kinds.as_ref().contains(&channel.kind))
            .collect())
    }

    #[tracing::instrument(skip_all)]
    async fn get_categories(&self) -> CommandResult<Vec<GuildChannel>> {
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
    async fn edit_channel(
        &self,
        category: &mut GuildChannel,
        definition: &GuildChannelDefinition,
    ) -> CommandResult<()> {
        if category.kind != definition.kind {
            anyhow::anyhow!("failed to edit category: kind is not matched");
        }

        category
            .edit(&self.discord_client, |edit| {
                edit.name(&definition.name)
                    .permissions(definition.permissions.clone())
            })
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(category = ?category))]
    async fn delete_channel(&self, category: &mut GuildChannel) -> CommandResult<()> {
        category.delete(&self.discord_client).await?;
        Ok(())
    }
}
