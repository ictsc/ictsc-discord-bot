use crate::bot::*;
use crate::services::redeploy::RedeployTarget;

use std::time::Duration;

use anyhow::Result;

use serenity::builder::{CreateApplicationCommand, CreateComponents};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::*;
use serenity::model::prelude::component::{ButtonStyle, ComponentType};
use serenity::prelude::*;

const CUSTOM_ID_REDEPLOY_CONFIRM: &str = "redeploy_confirm";
const CUSTOM_ID_REDEPLOY_CANCELED: &str = "redeploy_canceled";

#[derive(Debug, thiserror::Error)]
enum RedeployCommandError<'a> {
    #[error("問題コード `{0}` に対応する問題はありません。問題コードを再度お確かめください。")]
    InvalidProblemCodeError(&'a str),

    // /redeployコマンドの使用者のチームが解決できない時に発生するエラー
    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    UnexpectedSenderTeamsError,

    #[error("予期しないエラーが発生しました。")]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

type RedeployCommandResult<'t, T> = std::result::Result<T, RedeployCommandError<'t>>;

fn create_buttons(c: &mut CreateComponents, disabled: bool) -> &mut CreateComponents {
    c.create_action_row(|r| {
        r.create_button(|b| {
            b.style(ButtonStyle::Primary)
                .label("OK")
                .custom_id(CUSTOM_ID_REDEPLOY_CONFIRM)
                .disabled(disabled)
        })
        .create_button(|b| {
            b.style(ButtonStyle::Secondary)
                .label("キャンセル")
                .custom_id(CUSTOM_ID_REDEPLOY_CANCELED)
                .disabled(disabled)
        })
    })
}

impl Bot {
    pub fn create_redeploy_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name("redeploy")
            .description("問題環境を再展開します。")
            .create_option(|option| {
                option
                    .name("problem_code")
                    .description("問題コード")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    pub async fn handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let problem = match self.validate_redeploy_command(interaction) {
            Ok(problem) => problem,
            Err(err) => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(err.to_string())
                })
                .await?;
                return Ok(());
            }
        };

        self.defer_response(interaction).await?;

        if let Err(err) = self.do_redeploy_command(ctx, interaction, problem).await {
            tracing::error!(?err, "failed to do redeploy command");
            self.edit_response(interaction, |data| {
                data.content(err.to_string()).components(|c| c)
            })
            .await?;
        }

        Ok(())
    }

    fn validate_redeploy_command<'t>(
        &self,
        interaction: &'t ApplicationCommandInteraction,
    ) -> RedeployCommandResult<'t, &Problem> {
        // TODO: unwrapを治す
        let problem_code = self
            .get_option_as_str(interaction, "problem_code")
            .ok_or(anyhow::anyhow!("problem_code is not found"))
            .unwrap();

        let problem = self
            .problems
            .iter()
            .find(|problem| problem.code == problem_code);

        problem.ok_or(RedeployCommandError::InvalidProblemCodeError(problem_code))
    }

    // TODO: いろいろガバガバなので修正する
    async fn do_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
        problem: &Problem,
    ) -> RedeployCommandResult<()> {
        let sender = &interaction.user;
        let sender_member = self
            .guild_id
            .member(&self.discord_client, sender.id)
            .await
            .unwrap();

        let mut sender_teams = Vec::new();
        for role_id in sender_member.roles {
            let role = self.find_roles_by_id_cached(role_id).await.unwrap();
            match role {
                Some(role) => {
                    for team in &self.teams {
                        if role.name == team.role_name {
                            sender_teams.push(team);
                        }
                    }
                }
                None => (),
            }
        }

        // /joinコマンドの制約上、ユーザは高々1つのチームにしか所属しないはずである。
        // また、/redeployはGuildでのみ使用可能なため、チームに所属していないユーザは使用できない。
        if sender_teams.len() != 1 {
            return Err(RedeployCommandError::UnexpectedSenderTeamsError);
        }

        let sender_team = sender_teams.first().unwrap();

        self.edit_response(interaction, |data| {
            // TODO: チーム名にする
            data.content(format!(
                "チーム `{}` の問題 `{}` を再展開しますか？",
                sender_team.role_name, problem.name
            ))
            .components(|c| create_buttons(c, false))
        })
        .await
        .unwrap();

        let message = interaction
            .get_interaction_response(&self.discord_client)
            .await
            .unwrap();

        let component_interaction = message
            .await_component_interaction(ctx)
            .author_id(sender.id)
            .filter(|component_interaction| {
                component_interaction.data.component_type == ComponentType::Button
                    && (component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CONFIRM
                        || component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CANCELED)
            })
            .timeout(Duration::from_secs(60))
            .await;

        self.edit_response(interaction, |response| {
            response.components(|c| create_buttons(c, true))
        })
        .await
        .unwrap();

        let (component_interaction, should_recreate) = match component_interaction {
            Some(component_interaction) => {
                let should_recreate =
                    component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CONFIRM;
                (component_interaction, should_recreate)
            }
            None => {
                return Ok(());
            }
        };

        component_interaction
            .create_interaction_response(&self.discord_client, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
            .unwrap();

        if !should_recreate {
            component_interaction
                .edit_original_interaction_response(&self.discord_client, |response| {
                    response.content("再展開をやめました。")
                })
                .await
                .unwrap();
            return Ok(());
        }

        let target = RedeployTarget {
            team_id: sender_team.id.clone(),
            problem_id: problem.code.clone(),
        };
        let result = self.redeploy_service.redeploy(&target).await;

        match result {
            Ok(_) => {
                component_interaction
                    .edit_original_interaction_response(&self.discord_client, |response| {
                        response.content("再展開を開始しました。")
                    })
                    .await
                    .unwrap();
            }
            Err(_) => {
                component_interaction
                    .edit_original_interaction_response(&self.discord_client, |response| {
                        response.content("再展開を失敗しました。")
                    })
                    .await
                    .unwrap();
            }
        };

        for notifier in &self.redeploy_notifiers {
            notifier.notify(&target, &result).await;
        }

        Ok(())
    }
}
