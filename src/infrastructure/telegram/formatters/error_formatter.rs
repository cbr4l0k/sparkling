use crate::application::errors::ApplicationError;

pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format an error for user-friendly display
    pub fn format(error: &ApplicationError) -> String {
        match error {
            ApplicationError::NotFound(msg) => format!("Not found: {}", msg),
            ApplicationError::Unauthorized(msg) => format!("Access denied: {}", msg),
            ApplicationError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            ApplicationError::DomainError(e) => format!("Error: {}", e),
            ApplicationError::InternalError(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
        }
    }
}
