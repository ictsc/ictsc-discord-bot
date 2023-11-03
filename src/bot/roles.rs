use crate::bot::*;

use std::collections::HashMap;

use anyhow::Result;
use serenity::model::prelude::*;
use serenity::model::Permissions;

pub static EVERYONE_ROLE_NAME: &str = "@everyone";
pub static STAFF_ROLE_NAME: &str = "ICTSC2023 Staff";

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
    pub fn is_team_role(&self, role: &Role) -> bool {
        for team in &self.teams {
            if team.role_name == role.name {
                return true
            }
        }
        false
    }
}

// Roleに対するCRUD操作の実装
// READ権限のみは他のモジュールとも関連するため、publicにする
impl Bot {
    #[tracing::instrument(skip_all, fields(
        definition = ?definition,
    ))]
    async fn create_role(&self, definition: &RoleDefinition) -> Result<()> {
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

    #[tracing::instrument(skip_all)]
    pub async fn get_roles(&self) -> Result<Vec<Role>> {
        tracing::trace!("get roles");
        Ok(self
            .guild_id
            .roles(&self.discord_client)
            .await?
            .into_values()
            .collect())
    }

    #[tracing::instrument(skip_all, fields(
        role = ?role,
        definition = ?definition,
    ))]
    async fn edit_role(&self, role: &Role, definition: &RoleDefinition) -> Result<()> {
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
    async fn delete_role(&self, role: &Role) -> Result<()> {
        tracing::trace!("delete role called");
        self.guild_id
            .delete_role(&self.discord_client, role.id.0)
            .await?;
        Ok(())
    }
}

impl Bot {
    pub async fn update_role_cache(&self) -> Result<()> {
        tracing::trace!("update local role cache");
        let roles = self.get_roles().await?;
        let mut guard = self.role_cache.write().await;
        *guard = Some(roles);
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_roles_cached(&self) -> Result<Vec<Role>> {
        tracing::trace!("get roles cached");
        let guard = self.role_cache.read().await;
        match guard.as_ref() {
            Some(roles) => Ok(roles.clone()),
            None => Err(anyhow::anyhow!("role cache is not found")),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_role_map_cached(&self) -> Result<HashMap<String, Role>> {
        tracing::trace!("get role map cached");
        Ok(self
            .get_roles_cached()
            .await?
            .into_iter()
            .map(|role| (role.name.clone(), role))
            .collect())
    }

    #[tracing::instrument(skip_all, fields(
        name = ?name,
    ))]
    pub async fn find_roles_by_name_cached(&self, name: &str) -> Result<Vec<Role>> {
        tracing::trace!("find role by name cached");
        Ok(self
            .get_roles_cached()
            .await?
            .into_iter()
            .filter(|role| role.name == name)
            .collect())
    }

    pub async fn find_roles_by_id_cached(&self, id: RoleId) -> Result<Option<Role>> {
        tracing::trace!("find role by id cached");
        Ok(self
            .get_roles_cached()
            .await?
            .into_iter()
            .filter(|role| role.id == id)
            .nth(0))
    }
}

impl Bot {
    #[tracing::instrument(skip_all)]
    pub async fn sync_roles(&self) -> Result<()> {
        tracing::info!("sync roles");

        self.update_role_cache().await?;

        let mut definitions = Vec::new();

        definitions.push(
            RoleDefinitionBuilder::default()
                .name(EVERYONE_ROLE_NAME.to_string())
                .permissions(self.get_permissions_for_everyone())
                .mentionable(true)
                .build()?,
        );

        definitions.push(
            RoleDefinitionBuilder::default()
                .name(STAFF_ROLE_NAME.to_string())
                .permissions(self.get_permissions_for_staff())
                .colour(STAFF_ROLE_COLOUR)
                .hoist(true)
                .mentionable(true)
                .build()?,
        );

        for team in self.teams.iter() {
            definitions.push(
                RoleDefinitionBuilder::default()
                    .name(team.role_name.clone())
                    .permissions(self.get_permissions_for_team())
                    .hoist(true)
                    .mentionable(true)
                    .build()?,
            );
        }

        self._sync_roles(definitions).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_roles(&self) -> Result<()> {
        tracing::info!("delete all roles");
        self.update_role_cache().await?;
        self._sync_roles(&[]).await?;
        Ok(())
    }
}

impl Bot {
    // 与えられたRoleDefinitionのリストを基に、競技用ギルドのロールを同期する。
    // 実行前に、self.update_role_cache()を呼び出して、ロールキャッシュを更新しておく必要がある。
    async fn _sync_roles<T: AsRef<[RoleDefinition]>>(&self, definitions: T) -> Result<()> {
        tracing::debug!("fetch current roles");
        let roles = self.get_roles_cached().await?;

        tracing::debug!("sync defined roles");
        for definition in definitions.as_ref() {
            let matched_roles: Vec<_> =
                roles.iter().filter(|r| r.name == definition.name).collect();

            if matched_roles.len() == 1 {
                let role = matched_roles[0];
                if self.check_role_synced(role, definition) {
                    tracing::debug!(?role, "target role is created and synced, skip");
                    continue;
                }
                tracing::debug!(?role, "role is created, but is not synced, update role");
                self.edit_role(role, &definition).await?;
                continue;
            }

            if matched_roles.len() != 0 {
                tracing::debug!(
                    ?matched_roles,
                    "several matched roles are found, delete them"
                );
                for role in matched_roles {
                    self.delete_role(role).await?;
                }
            }

            tracing::debug!(?definition, "create role");
            self.create_role(&definition).await?
        }

        tracing::debug!("delete not-defined roles");
        for ref role in roles {
            let found = definitions
                .as_ref()
                .iter()
                .find(|d| d.name == role.name)
                .is_some();

            if !found {
                // @everyoneロールは必ず存在するため、削除対象から外す。
                // managedなロールは削除できない（integrationによって管理されている）ため、削除対象から外す
                if role.name == EVERYONE_ROLE_NAME || role.managed {
                    tracing::debug!(?role, "role can't delete it, skip");
                    continue;
                }

                tracing::debug!(?role, "role is not defined, delete it");
                self.delete_role(&role).await?;
            }
        }

        Ok(())
    }

    fn check_role_synced(&self, role: &Role, definition: &RoleDefinition) -> bool {
        role.name == definition.name
            && role.permissions == definition.permissions
            && role.colour.0 == definition.colour
            && role.hoist == definition.hoist
            && role.mentionable == definition.mentionable
    }
}
