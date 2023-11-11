use std::collections::HashSet;

use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::*;

use crate::bot::roles;
use crate::bot::Bot;

#[derive(Debug, thiserror::Error)]
enum JoinCommandError<'a> {
    #[error("このコマンドはDM以外から呼び出すことはできません。")]
    CalledFromGuildChannelError,
    #[error("招待コード `{0}` に対応するチームはありません。招待コードを再度お確かめください。")]
    InvalidInvitationCodeError(&'a str),
    #[error("ICTSC Discordチャンネルにまだ参加していません。参加した後に再度お試しください。")]
    UserNotInGuildError,

    #[error("予期しないエラーが発生しました。")]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

type JoinCommandResult<'t, T> = std::result::Result<T, JoinCommandError<'t>>;

impl Bot {
    pub fn create_join_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name("join")
            .description("チームに参加します。")
            .create_option(|option| {
                option
                    .name("invitation_code")
                    .description("招待コード")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    fn validate_join_command<'t>(
        &self,
        interaction: &'t ApplicationCommandInteraction,
    ) -> JoinCommandResult<'t, &str> {
        // joinコマンドはGlobalCommandなので、どこからでも呼び出すことは可能である。
        // だが、間違ってrandomチャンネル等で呼び出されてしまうことを防ぐため、DM以外からの呼び出しはエラーとする。
        if interaction.guild_id.is_some() {
            return Err(JoinCommandError::CalledFromGuildChannelError);
        }

        let invitation_code = self
            .get_option_as_str(interaction, "invitation_code")
            .unwrap();

        self.find_role_name_by_invitation_code(invitation_code)
            .ok_or(JoinCommandError::InvalidInvitationCodeError(
                invitation_code,
            ))
    }

    fn find_role_name_by_invitation_code(&self, invitation_code: &str) -> Option<&str> {
        // インフラパスが指定された場合、staff権限を付与する。
        if invitation_code == self.infra_password {
            return Some(roles::STAFF_ROLE_NAME);
        }

        // チームに割り当てられた招待コードの場合、チーム権限を付与する。
        for team in &self.teams {
            if team.invitation_code == invitation_code {
                return Some(&team.role_name);
            }
        }

        None
    }

    async fn do_join_command(
        &self,
        interaction: &ApplicationCommandInteraction,
        role_name: &str,
    ) -> JoinCommandResult<()> {
        // DMの送信元が、ICTSC Discordチャンネルに参加しているかをチェックする。
        let sender = &interaction.user;
        let mut sender_member = self
            .guild_id
            .member(&self.discord_client, sender.id)
            .await
            .map_err(|_| JoinCommandError::UserNotInGuildError)?;

        let sender_member_role_id_set = HashSet::from_iter(sender_member.roles.clone());

        let target_role_id_set: HashSet<_> = self
            .find_roles_by_name_cached(&role_name)
            .await
            .map_err(|err| JoinCommandError::Error(err.into()))?
            .into_iter()
            .map(|role| role.id)
            .collect();

        let role_ids_added: Vec<_> = target_role_id_set
            .difference(&sender_member_role_id_set)
            .map(|id| id.clone())
            .collect();

        let role_ids_removed: Vec<_> = sender_member_role_id_set
            .difference(&target_role_id_set)
            .map(|id| id.clone())
            .collect();

        sender_member
            .add_roles(&self.discord_client, &role_ids_added)
            .await
            .map_err(|err| JoinCommandError::Error(err.into()))?;

        sender_member
            .remove_roles(&self.discord_client, &role_ids_removed)
            .await
            .map_err(|err| JoinCommandError::Error(err.into()))?;

        self.edit_response(interaction, |data| {
            data.content(format!("チーム `{}` に参加しました。", role_name))
        })
        .await
        .map_err(|err| JoinCommandError::Error(err.into()))?;

        Ok(())
    }

    pub async fn handle_join_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let role_name = match self.validate_join_command(interaction) {
            Ok(role_name) => role_name,
            Err(err) => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(err.to_string())
                })
                .await?;
                return Ok(());
            },
        };

        tracing::trace!("send acknowledgement");
        self.defer_response(interaction).await?;

        if let Err(err) = self.do_join_command(interaction, role_name).await {
            tracing::error!(?err, "failed to do join command");
            self.edit_response(interaction, |data| data.content(err.to_string()))
                .await?;
        }

        Ok(())
    }
}
