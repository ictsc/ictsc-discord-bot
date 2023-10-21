use std::time::Duration;

use super::Bot;
use crate::*;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{command::*, InteractionResponseType, ReactionType};
use serenity::prelude::*;

const OK_REACTION: &str = "🙆\u{200d}♂\u{fe0f}";
const NG_REACTION: &str = "🙅\u{200d}♂\u{fe0f}";

impl Bot {
    pub fn create_recreate_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name("recreate")
            .description("問題環境を再作成します。")
            .create_option(|option| {
                option
                    .name("problem_code")
                    .description("問題コード")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    // TODO: いろいろガバガバなので修正する
    pub async fn handle_recreate_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let problem_code =
            InteractionHelper::value_of_as_str(interaction, "problem_code").unwrap();

        let sender = &interaction.user;

        let problem = self
            .problems
            .iter()
            .find(|problem| problem.code == problem_code);

        let problem = match problem {
            Some(problem) => problem,
            None => {
                interaction.create_interaction_response(&self.discord_client, |response| {
                    response.kind(InteractionResponseType::ChannelMessageWithSource);
                    response.interaction_response_data(|data| {
                        data.ephemeral(true).content(format!("問題コード `{}` に対応する問題はありません。", problem_code))
                    })
                }).await?;
                return Ok(());
            }
        };

        interaction.create_interaction_response(&self.discord_client, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource);
            response.interaction_response_data(|data| {
                data.content(format!("問題 `{}` を再作成しますか？", problem.name))
            })
        }).await?;

        let ok_reaction = ReactionType::Unicode(OK_REACTION.to_string());
        let ng_reaction = ReactionType::Unicode(NG_REACTION.to_string());

        let message = interaction.get_interaction_response(&self.discord_client).await?;

        message.react(&self.discord_client, ok_reaction).await?;
        message.react(&self.discord_client, ng_reaction).await?;

        let reaction = message
            .await_reaction(ctx)
            .author_id(sender.id)
            .added(true)
            .removed(false)
            .filter(|reaction| {
                if let ReactionType::Unicode(emoji) = &reaction.emoji {
                    emoji == OK_REACTION || emoji == NG_REACTION
                } else {
                    false
                }
            })
            .timeout(Duration::from_secs(30))
            .await;

        let reaction = match reaction {
            Some(reaction) => reaction,
            None => {
                message.reply(&self.discord_client, "タイムアウトしました。").await?;
                return Ok(());
            }
        };

        let should_be_recreated = match &reaction.as_inner_ref().emoji {
            ReactionType::Unicode(emoji) => {
                emoji == OK_REACTION
            },
            _ => {
                message.reply(&self.discord_client, "予期しない状態です").await?;
                return Ok(());
            }
        };

        if should_be_recreated {
            message.reply(&self.discord_client, "再作成を開始します。").await?;
        } else {
            message.reply(&self.discord_client, "再作成を中断します。").await?;
        }

        Ok(())
    }
}
