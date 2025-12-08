use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Entity '{entity}' with id '{id}' not found")]
    NotFound { entity: String, id: String },

    #[error("Invalid state: {message}")]
    InvalidState { message: String },

    #[error("Validation failed for field '{field}': {reason}")]
    ValidationFailed { field: String, reason: String },

    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),
}
