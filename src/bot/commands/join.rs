use super::Bot;
use crate::*;

use crate::{InteractionArgumentExtractor, InteractionDeferredResponder, InteractionHelper};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::*;

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

    pub async fn handle_join_command(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let invitation_code =
            InteractionHelper::value_of_as_str(interaction, "invitation_code").unwrap();

        let sender = &interaction.user;
        let mut sender_member = self
            .guild_id
            .member(&self.discord_client, sender.id)
            .await
            .map_err(|_| UserError::NotJoinedGuild)?;

        let role_name = self
            .find_role_name_by_invitation_code(invitation_code)
            .map_err(|_| UserError::InvalidInvitationCode(invitation_code.to_string()))?
            .ok_or(UserError::InvalidInvitationCode(
                invitation_code.to_string(),
            ))?;

        let sender_member_role_ids: Vec<_> = sender_member
            .roles
            .iter()
            .map(|role_id| role_id.clone())
            .collect();

        let target_role_ids: Vec<_> = self
            .find_roles_by_name(&role_name)
            .await
            .map_err(|err| SystemError::UnexpectedError(err.to_string()))?
            .into_iter()
            .map(|role| role.id)
            .collect();

        let role_ids_added: Vec<_> = target_role_ids
            .iter()
            .filter(|id| {
                !sender_member_role_ids
                    .iter()
                    .any(|sender_member_role_id| *id == sender_member_role_id)
            })
            .map(|id| id.clone())
            .collect();

        let role_ids_removed: Vec<_> = sender_member_role_ids
            .iter()
            .filter(|id| {
                !target_role_ids
                    .iter()
                    .any(|target_role_id| *id == target_role_id)
            })
            .map(|id| id.clone())
            .collect();

        sender_member
            .add_roles(&self.discord_client, &role_ids_added)
            .await
            .map_err(|err| SystemError::UnexpectedError(err.to_string()))?;

        sender_member
            .remove_roles(&self.discord_client, &role_ids_removed)
            .await
            .map_err(|err| SystemError::UnexpectedError(err.to_string()))?;

        InteractionHelper::defer_respond(
            &self.discord_client,
            &interaction,
            "チームに参加しました。",
        )
        .await?;
        Ok(())
    }

    fn find_role_name_by_invitation_code(&self, invitation_code: &str) -> Result<Option<String>> {
        // TODO: staffの招待はまた後で考える

        for team in &self.teams {
            if team.invitation_code == invitation_code {
                return Ok(Some(team.role_name.clone()));
            }
        }

        Ok(None)
    }
}
