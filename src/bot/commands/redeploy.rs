use std::time::Duration;

use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::builder::CreateComponents;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::component::ComponentType;
use serenity::model::user::User;
use serenity::prelude::*;

use crate::bot::helpers::HelperError;
use crate::bot::Bot;
use crate::models::Problem;
use crate::models::Team;
use crate::services::redeploy::RedeployTarget;

const CUSTOM_ID_REDEPLOY_CONFIRM: &str = "redeploy_confirm";
const CUSTOM_ID_REDEPLOY_CANCELED: &str = "redeploy_canceled";

#[derive(Debug, thiserror::Error)]
enum RedeployCommandError<'a> {
    #[error("問題コード `{0}` に対応する問題はありません。問題コードを再度お確かめください。")]
    InvalidProblemCodeError(&'a str),

    // /redeployコマンドの使用者のチームが解決できない時に発生するエラー
    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    UnexpectedSenderTeamsError,

    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    InconsistentCommandDefinitionError,

    #[error("予期しないエラーが発生しました。")]
    HelperError(#[from] HelperError),
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
            .description("問題環境の再展開に関するコマンド")
            .create_option(|option| {
                option
                    .name("start")
                    .description("問題環境を再展開します。")
                    .kind(CommandOptionType::SubCommand)
                    .create_sub_option(|option| {
                        option
                            .name("problem_code")
                            .description("問題コード")
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            })
            .create_option(|option| {
                option
                    .name("status")
                    .description("現在の再展開状況を表示します。")
                    .kind(CommandOptionType::SubCommand)
            })
    }

    pub async fn handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        if let Err(err) = self._handle_redeploy_command(ctx, interaction).await {
            tracing::error!(?err, "failed to handle redeploy command");
            self.edit_response(interaction, |data| {
                data.content(err.to_string()).components(|c| c)
            })
            .await?;
        }

        Ok(())
    }

    async fn _handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> RedeployCommandResult<()> {
        let subcommand = interaction
            .data
            .options
            .first()
            .ok_or(RedeployCommandError::InconsistentCommandDefinitionError)?;

        if subcommand.kind != CommandOptionType::SubCommand {
            return Err(RedeployCommandError::InconsistentCommandDefinitionError);
        }

        Ok(match subcommand.name.as_str() {
            "start" => {
                self.handle_redeploy_start_subcommand(ctx, interaction, subcommand)
                    .await?
            },
            "status" => self.handle_redeploy_status_subcommand(interaction).await?,
            _ => return Err(RedeployCommandError::InconsistentCommandDefinitionError),
        })
    }

    async fn get_team_for(&self, user: &User) -> RedeployCommandResult<Team> {
        let member = self.get_member(&user).await?;

        for role_id in member.roles {
            let role = self.find_roles_by_id_cached(role_id).await.unwrap();
            match role {
                Some(role) => {
                    for team in &self.teams {
                        if role.name == team.role_name {
                            return Ok(team.clone());
                        }
                    }
                },
                None => (),
            }
        }

        Err(RedeployCommandError::UnexpectedSenderTeamsError)
    }
}

impl Bot {
    async fn handle_redeploy_start_subcommand(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
        option: &CommandDataOption,
    ) -> RedeployCommandResult<()> {
        let problem = match self.validate_redeploy_start_subcommand(option) {
            Ok(problem) => problem,
            Err(err) => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(err.to_string())
                })
                .await?;
                return Ok(());
            },
        };

        self.defer_response(interaction).await?;

        if let Err(err) = self
            .do_redeploy_start_subcommand(ctx, interaction, problem)
            .await
        {
            tracing::error!(?err, "failed to do redeploy start subcommand");
            return Err(err);
        }

        Ok(())
    }

    fn validate_redeploy_start_subcommand<'t>(
        &self,
        option: &'t CommandDataOption,
    ) -> RedeployCommandResult<'t, &Problem> {
        let problem_code = self
            .get_option_as_str(&option.options, "problem_code")
            .unwrap();

        let problem = self
            .problems
            .iter()
            .find(|problem| problem.code == problem_code);

        problem.ok_or(RedeployCommandError::InvalidProblemCodeError(problem_code))
    }

    async fn do_redeploy_start_subcommand(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
        problem: &Problem,
    ) -> RedeployCommandResult<()> {
        let sender = &interaction.user;
        let sender_team = self.get_team_for(sender).await?;

        self.edit_response(interaction, |data| {
            // TODO: チーム名にする
            data.content(format!(
                "チーム `{}` の問題 `{}` を再展開しますか？",
                sender_team.role_name, problem.name
            ))
            .components(|c| create_buttons(c, false))
        })
        .await?;

        let message = self.get_response(interaction).await?;

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
        .await?;

        let (component_interaction, should_recreate) = match component_interaction {
            Some(component_interaction) => {
                let should_recreate =
                    component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CONFIRM;
                (component_interaction, should_recreate)
            },
            None => {
                return Ok(());
            },
        };
        let component_interaction = component_interaction.as_ref();

        self.defer_response(component_interaction).await?;

        if !should_recreate {
            self.edit_response(component_interaction, |response| {
                response.content("再展開をやめました。")
            })
            .await?;
            return Ok(());
        }

        let target = RedeployTarget {
            team_id: sender_team.id.clone(),
            problem_id: problem.code.clone(),
        };
        let result = self.redeploy_service.redeploy(&target).await;

        match result {
            Ok(_) => {
                self.edit_response(component_interaction, |response| {
                    response.content("再展開を開始しました。")
                })
                .await?;
            },
            Err(_) => {
                self.edit_response(component_interaction, |response| {
                    response.content("再展開に失敗しました。")
                })
                .await?;
            },
        };

        for notifier in &self.redeploy_notifiers {
            notifier.notify(&target, &result).await;
        }

        Ok(())
    }
}

impl Bot {
    async fn handle_redeploy_status_subcommand(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> RedeployCommandResult<()> {
        self.defer_response(interaction).await?;

        if let Err(err) = self.do_redeploy_status_subcommand(interaction).await {
            tracing::error!(?err, "failed to do redeploy status subcommand");
            return Err(err);
        }

        Ok(())
    }

    async fn do_redeploy_status_subcommand(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> RedeployCommandResult<()> {
        let sender = &interaction.user;
        let sender_team = self.get_team_for(sender).await?;

        let statuses = self
            .redeploy_service
            .get_status(&sender_team.id)
            .await
            .unwrap();

        let no_deploys = statuses
            .iter()
            .all(|status| status.last_redeploy_started_at.is_none());

        if no_deploys {
            self.edit_response(interaction, |data| {
                data.content("まだ再展開は実行されていません。")
            })
            .await?;
        }

        self.edit_response(interaction, |data| {
            data.embed(|e| {
                e.title("再展開状況");
                // TODO: 再展開状況はいい感じに表示する。今日はもう疲れた。
                for status in &statuses {
                    if status.last_redeploy_started_at.is_none() {
                        continue;
                    }

                    e.field(
                        &status.problem_code,
                        format!("{}", status.is_redeploying),
                        false,
                    );
                }
                e
            })
        })
        .await?;

        Ok(())
    }
}
