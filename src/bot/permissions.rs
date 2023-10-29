// This module manages entire permissions for ICTSC Discord channels.
use crate::bot::*;

use serenity::model::prelude::*;

impl Bot {
    // 全てのユーザに許可してよい権限
    pub fn get_permissions_for_everyone(&self) -> Permissions {
        Permissions::CHANGE_NICKNAME
    }

    // ICTSCスタッフに許可してよい権限
    pub fn get_permissions_for_staff(&self) -> Permissions {
        Permissions::all()
    }

    // 参加者に許可してよい権限
    pub fn get_permissions_for_team(&self) -> Permissions {
        Permissions::empty()
    }

    // Readonlyなpublic channelに設定される権限
    pub fn get_permissions_for_readonly_channel_member(&self) -> Permissions {
        Permissions::VIEW_CHANNEL | Permissions::READ_MESSAGE_HISTORY | Permissions::ADD_REACTIONS
    }

    // 投稿可能なpublic channelに設定される権限
    pub fn get_permissions_for_channel_member(&self) -> Permissions {
        self.get_permissions_for_readonly_channel_member()
            | Permissions::SEND_MESSAGES
            | Permissions::EMBED_LINKS
            | Permissions::USE_EXTERNAL_EMOJIS
            | Permissions::USE_EXTERNAL_STICKERS
            | Permissions::ADD_REACTIONS
    }

    // team channelに設定される権限
    pub fn get_permissions_for_team_channel_member(&self) -> Permissions {
        self.get_permissions_for_channel_member()
            | Permissions::STREAM
            | Permissions::ATTACH_FILES
            | Permissions::CONNECT
            | Permissions::SPEAK
            | Permissions::MUTE_MEMBERS
            | Permissions::DEAFEN_MEMBERS
            | Permissions::USE_VAD
            | Permissions::USE_SLASH_COMMANDS
            | Permissions::SEND_MESSAGES_IN_THREADS
    }

    // announceチャンネルに設定されるポリシー
    #[tracing::instrument(skip_all)]
    pub async fn get_permission_overwrites_for_announce_channel(
        &self,
    ) -> Result<Vec<PermissionOverwrite>> {
        tracing::trace!("get permission overrides for announce channel");

        let role_map = self.get_role_map().await?;

        let mut permissions = Vec::new();

        let everyone_role = role_map
            .get(roles::EVERYONE_ROLE_NAME)
            .ok_or(anyhow::anyhow!("@everyone role not found"))?;

        permissions.push(PermissionOverwrite {
            allow: self.get_permissions_for_readonly_channel_member(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(everyone_role.id),
        });

        Ok(permissions)
    }

    // randomチャンネルに設定されるポリシー
    #[tracing::instrument(skip_all)]
    pub async fn get_permission_overwrites_for_random_channel(
        &self,
    ) -> Result<Vec<PermissionOverwrite>> {
        tracing::trace!("get permission overrides for random channel");

        let role_map = self.get_role_map().await?;

        let mut permissions = Vec::new();

        let everyone_role = role_map
            .get(roles::EVERYONE_ROLE_NAME)
            .ok_or(anyhow::anyhow!("@everyone role not found"))?;

        permissions.push(PermissionOverwrite {
            allow: self.get_permissions_for_readonly_channel_member(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(everyone_role.id),
        });

        for team in &self.teams {
            let team_role = role_map
                .get(&team.role_name)
                .ok_or(anyhow::anyhow!("{} role not found", team.role_name))?;

            permissions.push(PermissionOverwrite {
                allow: self.get_permissions_for_channel_member(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(team_role.id),
            });
        }

        Ok(permissions)
    }

    // teamチャンネルに設定されるポリシー
    #[tracing::instrument(skip_all)]
    pub async fn get_permission_overwrites_for_team_channel(
        &self,
        team: &crate::bot::Team,
    ) -> Result<Vec<PermissionOverwrite>> {
        tracing::trace!("get permission overrides for random channel");

        let team_roles = self.find_roles_by_name(&team.role_name).await?;

        Ok(team_roles
            .into_iter()
            .map(|role| PermissionOverwrite {
                allow: self.get_permissions_for_team_channel_member(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(role.id),
            })
            .collect())
    }
}
