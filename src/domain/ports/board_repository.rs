use async_trait::async_trait;
use crate::domain::entities::{Board, Column};
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

/// Port for board repository operations
#[async_trait]
pub trait BoardRepository: Send + Sync {
    /// Find a board by ID
    async fn find_by_id(
        &self,
        account_id: &FizzyId,
        id: &FizzyId,
    ) -> Result<Option<Board>, DomainError>;

    /// Find a board by name (case-insensitive)
    async fn find_by_name(
        &self,
        account_id: &FizzyId,
        name: &str,
    ) -> Result<Option<Board>, DomainError>;

    /// List all boards accessible to a user
    async fn list_accessible(
        &self,
        account_id: &FizzyId,
        user_id: &FizzyId,
    ) -> Result<Vec<Board>, DomainError>;

    /// Get all columns for a board
    async fn get_columns(
        &self,
        account_id: &FizzyId,
        board_id: &FizzyId,
    ) -> Result<Vec<Column>, DomainError>;

    /// Check if user has access to board
    async fn user_has_access(
        &self,
        account_id: &FizzyId,
        board_id: &FizzyId,
        user_id: &FizzyId,
    ) -> Result<bool, DomainError>;
}
