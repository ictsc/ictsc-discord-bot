use std::collections::HashSet;

use anyhow::Result;
use serenity::all::CommandInteraction;
use serenity::all::CreateCommand;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::EditInteractionResponse;

use crate::bot::helpers::HelperError;
use crate::bot::Bot;

#[derive(Debug, thiserror::Error)]
enum SyncCommandError {
    #[error("このコマンドはDM以外から呼び出すことはできません。")]
    CalledFromGuildChannelError,
    #[error("ICTSC Discordチャンネルにまだ参加していません。参加した後に再度お試しください。")]
    UserNotInGuildError,
    #[error("チームのroleが見つかりませんでした。運営にお問い合わせください。")]
    TeamNotFoundError,

    #[error("予期しないエラーが発生しました。")]
    HelperError(#[from] HelperError),
}

type SyncCommandResult<'t, T> = std::result::Result<T, SyncCommandError>;

impl Bot {
    pub fn create_sync_command() -> CreateCommand {
        CreateCommand::new("sync")
            .description("スコアサーバーのユーザー情報からDiscordロールを付与します。")
    }

    #[tracing::instrument(skip_all)]
    pub async fn handle_sync_command(&self, interaction: &CommandInteraction) -> Result<()> {
        if interaction.guild_id.is_some() {
            return Err(SyncCommandError::CalledFromGuildChannelError.into());
        }

        let contestatnt = self
            .contestant_service
            .get_contestant(&interaction.user.id.to_string())
            .await;
        let role_name = match contestatnt {
            Ok(c) => self
                .teams
                .iter()
                .find_map(|t| (c.team_id == t.id).then(|| t.role_name.clone()))
                .ok_or(SyncCommandError::TeamNotFoundError)?,
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

        if let Err(err) = self.do_sync_command(interaction, &role_name).await {
            tracing::error!(?err, "failed to do sync command");
            self.edit_response(
                interaction,
                EditInteractionResponse::new().content(err.to_string()),
            )
            .await?;
        }

        Ok(())
    }

    async fn do_sync_command(
        &self,
        interaction: &CommandInteraction,
        role_name: &str,
    ) -> SyncCommandResult<()> {
        // DMの送信元が、ICTSC Discordチャンネルに参加しているかをチェックする。
        let sender = &interaction.user;
        let mut sender_member = self
            .get_member(sender)
            .await
            .map_err(|_| SyncCommandError::UserNotInGuildError)?;

        let sender_member_role_id_set = HashSet::from_iter(sender_member.roles.clone());

        let target_role_id_set: HashSet<_> = self
            .find_roles_by_name_cached(&role_name)
            .await?
            .iter()
            .map(|role| role.id)
            .collect();

        let role_ids_granted: Vec<_> = target_role_id_set
            .difference(&sender_member_role_id_set)
            .map(|id| id.clone())
            .collect();

        let role_ids_revoked: Vec<_> = sender_member_role_id_set
            .difference(&target_role_id_set)
            .map(|id| id.clone())
            .collect();

        self.grant_roles(&mut sender_member, role_ids_granted)
            .await?;

        self.revoke_roles(&mut sender_member, role_ids_revoked)
            .await?;

        self.edit_response(
            interaction,
            EditInteractionResponse::new()
                .content(format!("チーム `{}` のロールを付与しました。", role_name)),
        )
        .await?;

        Ok(())
    }
}
