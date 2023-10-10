use serenity::model::prelude::*;
use serenity::model::Permissions;
use crate::CommandResult;

use super::Bot;

static EVERYONE_ROLE_NAME: &str = "@everyone";
static STAFF_ROLE_NAME: &str = "ICTSC2023 Staff";
static STAFF_ROLE_COLOUR: u32 = 14942278;

#[derive(Clone, Debug, derive_builder::Builder)]
struct RoleDefinition {
    name: String,
    permissions: Permissions,
    #[builder(default)]
    colour: u32,
    #[builder(default)]
    hoist: bool,
    #[builder(default)]
    mentionable: bool,
}

impl Bot {
    #[tracing::instrument(skip_all)]
    pub async fn sync_roles(&self) -> CommandResult<()> {
        tracing::info!("sync roles");

        let mut roles = Vec::new();

        roles.push(RoleDefinitionBuilder::default()
            .name(EVERYONE_ROLE_NAME.to_string())
            .permissions(Permissions::empty())
            .mentionable(false)
            .build()?
        );

        roles.push(RoleDefinitionBuilder::default()
            .name(STAFF_ROLE_NAME.to_string())
            .permissions(Permissions::all())
            .colour(STAFF_ROLE_COLOUR)
            .hoist(true)
            .mentionable(true)
            .build()?
        );

        self._sync_roles(roles).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_roles(&self) -> CommandResult<()> {
        tracing::info!("delete all roles");
        self._sync_roles(&[]).await?;
        Ok(())
    }
}

impl Bot {
    #[tracing::instrument(skip_all)]
    async fn _sync_roles<T: AsRef<[RoleDefinition]>>(&self, definitions: T) -> CommandResult<()> {
        tracing::debug!("fetch current roles");
        let roles = self.discord_client
            .get_guild_roles(self.guild_id.0)
            .await?;

        tracing::debug!("sync defined roles");
        for definition in definitions.as_ref() {
            let matched_roles: Vec<_> = roles.iter()
                .filter(|r| r.name == definition.name)
                .collect();

            if matched_roles.len() == 1 {
                let role = matched_roles[0];
                if self.check_role_synced(role, definition) {
                    tracing::debug!(?role, "target role is created and synced, skip");
                    continue
                }
                tracing::debug!(?role, "role is created, but is not synced, update role");
                self.edit_role(role, &definition).await?;
                continue;
            }

            if matched_roles.len() != 0 {
                tracing::debug!(?matched_roles, "several matched roles are found, delete them");
                for role in matched_roles {
                    self.delete_role(role).await?;
                }
            }

            tracing::debug!(?definition, "create role");
            self.create_role(&definition).await?
        }

        tracing::debug!("delete not-defined roles");
        for ref role in roles {
            let found = definitions.as_ref()
                .iter()
                .find(|d| d.name == role.name)
                .is_some();

            if !found {
                // @everyoneロールは必ず存在するため、削除対象から外す。
                // managedなロールは削除できない（integrationによって管理されている）ため、削除対象から外す
                if role.name == EVERYONE_ROLE_NAME || role.managed {
                    tracing::debug!(?role, "role can't delete it, skip");
                    continue
                }

                tracing::debug!(?role, "role is not defined, delete it");
                self.delete_role(&role).await?;
            }
        }

        Ok(())
    }

    fn check_role_synced(&self, role: &Role, definition: &RoleDefinition) -> bool {
        role.name == definition.name &&
            role.permissions == definition.permissions &&
            role.colour.0 == definition.colour &&
            role.hoist == definition.hoist &&
            role.mentionable == definition.mentionable
    }

    #[tracing::instrument(skip_all, fields(
        definition = ?definition,
    ))]
    async fn create_role(&self, definition: &RoleDefinition) -> CommandResult<()> {
        tracing::trace!("create role called");
        let definition = definition.clone();
        self.guild_id
            .create_role(&self.discord_client, |edit| {
                edit.name(definition.name)
                    .permissions(definition.permissions)
                    .colour(definition.colour as u64)
                    .hoist(definition.hoist)
                    .mentionable(definition.mentionable)
            })
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(
        role = ?role,
        definition = ?definition,
    ))]
    async fn edit_role(&self, role: &Role, definition: &RoleDefinition) -> CommandResult<()> {
        tracing::trace!("edit role called");
        let definition = definition.clone();
        self.guild_id
            .edit_role(&self.discord_client, role.id.0, |edit| {
                edit.name(definition.name)
                    .permissions(definition.permissions)
                    .colour(definition.colour as u64)
                    .hoist(definition.hoist)
                    .mentionable(definition.mentionable)
            })
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(
        role = ?role,
    ))]
    async fn delete_role(&self, role: &Role) -> CommandResult<()> {
        tracing::trace!("delete role called");
        self.guild_id
            .delete_role(&self.discord_client, role.id.0)
            .await?;
        Ok(())
    }
}