use std::collections::HashSet;

use anyhow::Result;
use serenity::all::CommandInteraction;
use serenity::all::CommandOptionType;
use serenity::all::CreateCommand;
use serenity::all::CreateCommandOption;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::EditInteractionResponse;

use crate::bot::helpers::HelperError;
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
    HelperError(#[from] HelperError),
}

type JoinCommandResult<'t, T> = std::result::Result<T, JoinCommandError<'t>>;

impl Bot {
    pub fn create_join_command() -> CreateCommand {
        CreateCommand::new("join")
            .description("チームに参加します。")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "invitation_code",
                    "招待コード",
                )
                .required(true),
            )
    }

    #[tracing::instrument(skip_all)]
    pub async fn handle_join_command(&self, interaction: &CommandInteraction) -> Result<()> {
        let role_name = match self.validate_join_command(interaction) {
            Ok(role_name) => role_name,
            Err(err) => {
                self.respond(
                    interaction,
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content(err.to_string()),
                )
                .await?;
                return Ok(());
            },
        };

        tracing::trace!("send acknowledgement");
        self.defer_response(interaction).await?;

        if let Err(err) = self.do_join_command(interaction, role_name).await {
            tracing::error!(?err, "failed to do join command");
            self.edit_response(
                interaction,
                EditInteractionResponse::new().content(err.to_string()),
            )
            .await?;
        }

        Ok(())
    }

    fn validate_join_command<'t>(
        &self,
        interaction: &'t CommandInteraction,
    ) -> JoinCommandResult<'t, &str> {
        // joinコマンドはGlobalCommandなので、どこからでも呼び出すことは可能である。
        // だが、間違ってrandomチャンネル等で呼び出されてしまうことを防ぐため、DM以外からの呼び出しはエラーとする。
        if interaction.guild_id.is_some() {
            return Err(JoinCommandError::CalledFromGuildChannelError);
        }

        let invitation_code = self
            .get_option_as_str(&interaction.data.options, "invitation_code")
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
        interaction: &CommandInteraction,
        role_name: &str,
    ) -> JoinCommandResult<()> {
        // DMの送信元が、ICTSC Discordチャンネルに参加しているかをチェックする。
        let sender = &interaction.user;
        let mut sender_member = self
            .get_member(sender)
            .await
            .map_err(|_| JoinCommandError::UserNotInGuildError)?;

        let sender_member_role_id_set = HashSet::from_iter(sender_member.roles.clone());

        let target_role_id_set: HashSet<_> = self
            .find_roles_by_name_cached(role_name)
            .await?
            .iter()
            .map(|role| role.id)
            .collect();

        let role_ids_granted: Vec<_> = target_role_id_set
            .difference(&sender_member_role_id_set).copied()
            .collect();

        let role_ids_revoked: Vec<_> = sender_member_role_id_set
            .difference(&target_role_id_set).copied()
            .collect();

        self.grant_roles(&mut sender_member, role_ids_granted)
            .await?;

        self.revoke_roles(&mut sender_member, role_ids_revoked)
            .await?;

        self.edit_response(
            interaction,
            EditInteractionResponse::new()
                .content(format!("チーム `{}` に参加しました。", role_name)),
        )
        .await?;

        Ok(())
    }
}
