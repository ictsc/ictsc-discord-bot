pub mod ask;
pub mod join;
pub mod recreate;

pub struct ApplicationCommandContext {
    pub context: serenity::prelude::Context,
    pub command: serenity::model::prelude::application_command::ApplicationCommandInteraction,
}

pub struct ReactionContext {
    pub context: serenity::prelude::Context,
    pub reaction: serenity::model::prelude::Reaction,
}
