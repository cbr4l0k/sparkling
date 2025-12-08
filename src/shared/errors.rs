use thiserror::Error;
use crate::application::errors::ApplicationError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Application(#[from] ApplicationError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Telegram error: {0}")]
    Telegram(String),
}
