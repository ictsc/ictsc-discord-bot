pub mod channels;
pub mod interactions;
pub mod roles;

#[derive(Debug, thiserror::Error)]
pub enum HelperError {
    #[error("channel kind is invalid")]
    InvalidChannelKindError,

    #[error("role cache is not populated")]
    RoleCacheNotPopulatedError,

    #[error("{0}")]
    SerenityError(#[from] serenity::Error),
}

pub type HelperResult<T> = std::result::Result<T, HelperError>;
