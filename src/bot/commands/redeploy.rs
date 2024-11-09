use std::time::Duration;

use anyhow::Result;
use serenity::all::ButtonStyle;
use serenity::all::CommandDataOption;
use serenity::all::CommandDataOptionValue;
use serenity::all::CommandInteraction;
use serenity::all::CommandOptionType;
use serenity::all::ComponentInteractionDataKind;
use serenity::all::CreateActionRow;
use serenity::all::CreateButton;
use serenity::all::CreateCommand;
use serenity::all::CreateCommandOption;
use serenity::all::CreateEmbed;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::EditInteractionResponse;
use serenity::model::user::User;
use serenity::prelude::*;

use crate::bot::helpers::HelperError;
use crate::bot::Bot;
use crate::models::Problem;
use crate::models::Team;
use crate::services::redeploy::RedeployError;
use crate::services::redeploy::RedeployTarget;

const CUSTOM_ID_REDEPLOY_CONFIRM: &str = "redeploy_confirm";
const CUSTOM_ID_REDEPLOY_CANCELED: &str = "redeploy_canceled";

#[derive(Debug, thiserror::Error)]
enum RedeployCommandError<'a> {
    #[error("問題コード `{0}` に対応する問題はありません。問題コードを再度お確かめください。")]
    InvalidProblemCodeError(&'a str),

    #[error("問題 `{0}` の再展開は実行中です。再展開が完了してから再度お試しください。")]
    AnotherJobInQueue(String),

    // /redeployコマンドの使用者のチームが解決できない時に発生するエラー
    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    UnexpectedSenderTeamsError,

    // redeploy serviceからエラーが帰ってきた時に発生するエラー
    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    RedeployServiceError(#[from] RedeployError),

    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    InconsistentCommandDefinitionError,

    #[error("予期しないエラーが発生しました。")]
    HelperError(#[from] HelperError),
}

type RedeployCommandResult<'t, T> = std::result::Result<T, RedeployCommandError<'t>>;

fn create_buttons(disabled: bool) -> Vec<CreateActionRow> {
    let ok = CreateButton::new(CUSTOM_ID_REDEPLOY_CONFIRM)
        .label("OK")
        .style(ButtonStyle::Primary)
        .disabled(disabled);

    let cancel = CreateButton::new(CUSTOM_ID_REDEPLOY_CANCELED)
        .label("キャンセル")
        .style(ButtonStyle::Secondary)
        .disabled(disabled);

    vec![CreateActionRow::Buttons(vec![ok, cancel])]
}

impl Bot {
    pub fn create_redeploy_command() -> CreateCommand {
        CreateCommand::new("redeploy")
            .description("問題環境の再展開に関するコマンド")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "start",
                    "問題環境を再展開します。",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "problem_code",
                        "問題コード",
                    )
                    .required(true),
                ),
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "status",
                "現在の再展開状況を表示します。",
            ))
    }

    #[tracing::instrument(skip_all)]
    pub async fn handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        if let Err(err) = self._handle_redeploy_command(ctx, interaction).await {
            tracing::error!(?err, "failed to handle redeploy command");
            self.edit_response(
                interaction,
                EditInteractionResponse::new().content(err.to_string()),
            )
            .await?;
        }

        Ok(())
    }

    async fn _handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> RedeployCommandResult<()> {
        let subcommand = interaction
            .data
            .options
            .first()
            .ok_or(RedeployCommandError::InconsistentCommandDefinitionError)?;

        match &subcommand.value {
            CommandDataOptionValue::SubCommand(options) => match subcommand.name.as_str() {
                "start" => {
                    self.handle_redeploy_start_subcommand(ctx, interaction, options)
                        .await?
                },
                "status" => self.handle_redeploy_status_subcommand(interaction).await?,
                _ => return Err(RedeployCommandError::InconsistentCommandDefinitionError),
            },
            _ => return Err(RedeployCommandError::InconsistentCommandDefinitionError),
        }

        Ok(())
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
    #[tracing::instrument(skip_all)]
    async fn handle_redeploy_start_subcommand(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
        options: &[CommandDataOption],
    ) -> RedeployCommandResult<()> {
        let problem = match self.validate_redeploy_start_subcommand(options) {
            Ok(problem) => problem,
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
        options: &'t [CommandDataOption],
    ) -> RedeployCommandResult<'t, &Problem> {
        let problem_code = self.get_option_as_str(options, "problem_code").unwrap();

        // スコアサーバーとの互換性のため、ここで大文字に正規化する
        let normalized_problem_code = problem_code.to_uppercase();

        let problem = self
            .problems
            .iter()
            .find(|problem| problem.code == normalized_problem_code);

        problem.ok_or(RedeployCommandError::InvalidProblemCodeError(problem_code))
    }

    async fn do_redeploy_start_subcommand(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
        problem: &Problem,
    ) -> RedeployCommandResult<()> {
        let sender = &interaction.user;
        let sender_team = self.get_team_for(sender).await?;

        let redeploy_status = self.redeploy_service.get_status(&sender_team.id).await?;
        let redeploy_job_exists = redeploy_status.iter().any(|status| {
            // リクエストされた問題が既に再展開中か？
            status.problem_code == problem.code
                && status.last_redeploy_started_at.is_some()
                && status.last_redeploy_completed_at.is_none()
        });

        if redeploy_job_exists {
            return Err(RedeployCommandError::AnotherJobInQueue(
                problem.name.clone(),
            ));
        }

        self.edit_response(
            interaction,
            EditInteractionResponse::new()
                .content(format!(
                    "チーム `{}` の問題 `{}` を再展開しますか？",
                    sender_team.role_name, problem.name
                ))
                .components(create_buttons(false)),
        )
        .await?;

        let message = self.get_response(interaction).await?;

        let component_interaction = message
            .await_component_interaction(ctx)
            .author_id(sender.id)
            .filter(
                |component_interaction| match &component_interaction.data.kind {
                    ComponentInteractionDataKind::Button => {
                        component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CONFIRM
                            || component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CANCELED
                    },
                    _ => false,
                },
            )
            .timeout(Duration::from_secs(60))
            .await;

        let component_interaction = match component_interaction {
            Some(component_interaction) => component_interaction,
            None => {
                self.edit_response(
                    interaction,
                    EditInteractionResponse::new()
                        .content("タイムアウトしました。再度、再作成リクエストを投稿してください。")
                        .components(create_buttons(true)),
                )
                .await?;
                return Ok(());
            },
        };

        self.edit_response(
            interaction,
            EditInteractionResponse::new().components(create_buttons(true)),
        )
        .await?;
        self.defer_response(&component_interaction).await?;

        let should_recreate = component_interaction.data.custom_id == CUSTOM_ID_REDEPLOY_CONFIRM;
        if !should_recreate {
            self.edit_response(
                &component_interaction,
                EditInteractionResponse::new().content("再展開を中止しました。"),
            )
            .await?;
            return Ok(());
        }

        let target = RedeployTarget {
            team_id: sender_team.id.clone(),
            problem_id: problem.code.clone(),
        };
        let result = self.redeploy_service.redeploy(&target).await;

        match &result {
            Ok(_) => {
                self.edit_response(
                    &component_interaction,
                    EditInteractionResponse::new().content("再展開を開始しました。"),
                )
                .await?;
            },
            Err(err) => match err {
                RedeployError::AnotherJobInQueue(_) => {
                    self.edit_response(
                        &component_interaction,
                        EditInteractionResponse::new() .content("この問題は既に再展開リクエストが投げられています。再展開が完了してから再度お試しください。")
                    )
                    .await?;
                },

                _ => {
                    tracing::error!(?err, "failed to redeploy");
                    self.edit_response(
                        &component_interaction,
                        EditInteractionResponse::new().content(
                            "再展開中にエラーが発生しました。運営にお問い合わせください。",
                        ),
                    )
                    .await?;
                },
            },
        };

        for notifier in &self.redeploy_notifiers {
            notifier.notify(&target, &result).await;
        }

        Ok(())
    }
}

impl Bot {
    #[tracing::instrument(skip_all)]
    async fn handle_redeploy_status_subcommand(
        &self,
        interaction: &CommandInteraction,
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
        interaction: &CommandInteraction,
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
            self.edit_response(
                interaction,
                EditInteractionResponse::new().content("まだ再展開は実行されていません。"),
            )
            .await?;
            return Ok(());
        }

        let mut embed = CreateEmbed::new().title("再展開状況");
        for status in &statuses {
            let started_at = match status.last_redeploy_started_at {
                Some(started_at) => started_at,
                None => continue,
            };

            let name = status.problem_code.clone();
            let problem_name = self
                .problems
                .iter()
                .find(|problem| problem.code == status.problem_code)
                .map(|problem| format!("{}: {}", name, problem.name))
                .unwrap_or_else(|| name);

            let value = match status.last_redeploy_completed_at {
                Some(completed_at) => {
                    let completed_at_local = completed_at.with_timezone(&chrono_tz::Asia::Tokyo);
                    format!(
                        "🎉 再展開完了（完了時刻：{}）",
                        completed_at_local.format("%Y/%m/%d %H:%M:%S")
                    )
                },
                None => {
                    let started_at_local = started_at.with_timezone(&chrono_tz::Asia::Tokyo);
                    format!(
                        "⚙️ 再展開中（開始時刻：{}）",
                        started_at_local.format("%Y/%m/%d %H:%M:%S")
                    )
                },
            };

            embed = embed.field(problem_name, value, false);
        }

        self.edit_response(interaction, EditInteractionResponse::new().add_embed(embed))
            .await?;

        Ok(())
    }
}
