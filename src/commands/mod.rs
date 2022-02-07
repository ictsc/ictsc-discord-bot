pub mod ask;
pub mod join;
pub mod whoami;

pub struct ApplicationCommandContext {
    pub context: serenity::prelude::Context,
    pub command: serenity::model::prelude::application_command::ApplicationCommandInteraction,
}
