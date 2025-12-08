use std::sync::Arc;
use crate::domain::entities::Comment;
use crate::domain::ports::{CommentRepository, CardRepository, EventRepository, CreateEventInput, event_actions};
use crate::domain::value_objects::FizzyId;
use crate::application::errors::ApplicationError;

pub struct AddCommentUseCase {
    comment_repository: Arc<dyn CommentRepository>,
    card_repository: Arc<dyn CardRepository>,
    event_repository: Arc<dyn EventRepository>,
}

pub struct AddCommentInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub card_number: i64,
    pub content: String,
}

impl AddCommentUseCase {
    pub fn new(
        comment_repository: Arc<dyn CommentRepository>,
        card_repository: Arc<dyn CardRepository>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self { comment_repository, card_repository, event_repository }
    }

    pub async fn execute(&self, input: AddCommentInput) -> Result<Comment, ApplicationError> {
        if input.content.trim().is_empty() {
            return Err(ApplicationError::InvalidInput("Comment cannot be empty".to_string()));
        }

        let card = self.card_repository
            .find_by_number(&input.account_id, input.card_number)
            .await
            .map_err(ApplicationError::DomainError)?
            .ok_or_else(|| ApplicationError::NotFound(format!("Card #{} not found", input.card_number)))?;

        let comment = self.comment_repository
            .create(&input.account_id, &card.id, &input.user_id, &input.content)
            .await
            .map_err(ApplicationError::DomainError)?;

        // Create event
        let _ = self.event_repository
            .create_event(&input.account_id, CreateEventInput {
                board_id: card.board_id.clone(),
                eventable_id: comment.id.clone(),
                eventable_type: "Comment".to_string(),
                creator_id: input.user_id,
                action: event_actions::COMMENT_CREATED.to_string(),
                particulars: serde_json::json!({
                    "card_id": card.id.as_str()
                }),
            })
            .await;

        Ok(comment)
    }
}
