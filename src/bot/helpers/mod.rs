pub(crate) mod channels;

#[derive(Debug, thiserror::Error)]
pub(crate) enum HelperError {
    #[error("{0}")]
    SerenityError(#[from] serenity::Error),
}

pub(crate) type HelperResult<T> = std::result::Result<T, HelperError>;
