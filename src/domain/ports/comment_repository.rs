use async_trait::async_trait;
use crate::domain::entities::Comment;
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

/// Port for comment repository operations
#[async_trait]
pub trait CommentRepository: Send + Sync {
    /// List comments for a card (most recent first)
    async fn list_for_card(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
        limit: Option<i64>,
    ) -> Result<Vec<Comment>, DomainError>;

    /// Add a comment to a card
    async fn create(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
        creator_id: &FizzyId,
        content: &str,
    ) -> Result<Comment, DomainError>;
}
