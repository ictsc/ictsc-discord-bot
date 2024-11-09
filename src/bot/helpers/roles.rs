use std::collections::HashMap;

use serenity::all::EditRole;
use serenity::model::prelude::Role;
use serenity::model::prelude::RoleId;
use serenity::model::Permissions;

use super::HelperError;
use super::HelperResult;
use crate::bot::Bot;

#[derive(Clone, Debug, derive_builder::Builder)]
pub struct RoleDefinition {
    pub name: String,
    pub permissions: Permissions,
    #[builder(default)]
    pub colour: u32,
    #[builder(default)]
    pub hoist: bool,
    #[builder(default)]
    pub mentionable: bool,
}

// Guildのロールを操作するためのヘルパー関数
impl Bot {
    #[tracing::instrument(skip_all, fields(
        definition = ?definition,
    ))]
    pub async fn create_role(&self, definition: &RoleDefinition) -> HelperResult<Role> {
        tracing::trace!("Create role");
        let definition = definition.clone();
        Ok(self
            .guild_id
            .create_role(
                &self.discord_client,
                EditRole::new()
                    .name(definition.name)
                    .permissions(definition.permissions)
                    .colour(definition.colour)
                    .hoist(definition.hoist)
                    .mentionable(definition.mentionable),
            )
            .await?)
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_roles(&self) -> HelperResult<Vec<Role>> {
        tracing::trace!("Get roles");
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
    pub async fn edit_role(&self, role: &Role, definition: &RoleDefinition) -> HelperResult<Role> {
        tracing::trace!("Edit role called");
        let definition = definition.clone();
        Ok(self
            .guild_id
            .edit_role(
                &self.discord_client,
                role.id,
                EditRole::new()
                    .name(definition.name)
                    .permissions(definition.permissions)
                    .colour(definition.colour)
                    .hoist(definition.hoist)
                    .mentionable(definition.mentionable),
            )
            .await?)
    }

    #[tracing::instrument(skip_all, fields(role = ?role))]
    pub async fn delete_role(&self, role: &Role) -> HelperResult<()> {
        tracing::trace!("Delete role called");
        Ok(self
            .guild_id
            .delete_role(&self.discord_client, role.id)
            .await?)
    }
}

impl Bot {
    #[tracing::instrument(skip_all)]
    pub async fn update_role_cache(&self) -> HelperResult<()> {
        tracing::trace!("Update role cache");
        let roles = self.get_roles().await?;
        let mut guard = self.role_cache.write().await;
        *guard = Some(roles);
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_roles_cached(&self) -> HelperResult<Vec<Role>> {
        tracing::trace!("Get roles (cached)");
        let guard = self.role_cache.read().await;
        match guard.as_ref() {
            Some(roles) => Ok(roles.clone()),
            None => Err(HelperError::RoleCacheNotPopulatedError),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_role_map_cached(&self) -> HelperResult<HashMap<String, Role>> {
        tracing::trace!("Get role map (cached)");
        Ok(self
            .get_roles_cached()
            .await?
            .into_iter()
            .map(|role| (role.name.clone(), role))
            .collect())
    }

    #[tracing::instrument(skip_all, fields(id = ?id))]
    pub async fn find_roles_by_id_cached(&self, id: RoleId) -> HelperResult<Option<Role>> {
        tracing::trace!("Find role by id (cached)");
        Ok(self
            .get_roles_cached()
            .await?
            .into_iter()
            .filter(|role| role.id == id)
            .nth(0))
    }

    #[tracing::instrument(skip_all, fields(name = ?name))]
    pub async fn find_roles_by_name_cached(&self, name: &str) -> HelperResult<Vec<Role>> {
        tracing::trace!("Find role by name (cached)");
        Ok(self
            .get_roles_cached()
            .await?
            .into_iter()
            .filter(|role| role.name == name)
            .collect())
    }
}
