use std::sync::Arc;
use crate::domain::entities::Card;
use crate::domain::ports::{CardRepository, CreateCardInput, BoardRepository, EventRepository, CreateEventInput, event_actions};
use crate::domain::value_objects::{FizzyId, CardStatus};
use crate::application::errors::ApplicationError;

pub struct CreateCardUseCase {
    card_repository: Arc<dyn CardRepository>,
    board_repository: Arc<dyn BoardRepository>,
    event_repository: Arc<dyn EventRepository>,
}

pub struct CreateCardUseCaseInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub board_id: FizzyId,
    pub title: String,
    pub description: Option<String>,
}

impl CreateCardUseCase {
    pub fn new(
        card_repository: Arc<dyn CardRepository>,
        board_repository: Arc<dyn BoardRepository>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self { card_repository, board_repository, event_repository }
    }

    pub async fn execute(&self, input: CreateCardUseCaseInput) -> Result<Card, ApplicationError> {
        // Verify access
        let has_access = self.board_repository
            .user_has_access(&input.account_id, &input.board_id, &input.user_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        if !has_access {
            return Err(ApplicationError::Unauthorized("No access to this board".to_string()));
        }

        let create_input = CreateCardInput {
            board_id: input.board_id.clone(),
            creator_id: input.user_id.clone(),
            title: input.title,
            description: input.description,
            status: CardStatus::Drafted,
            column_id: None,
        };

        let card = self.card_repository
            .create(&input.account_id, create_input)
            .await
            .map_err(ApplicationError::DomainError)?;

        // Create event for audit trail
        let _ = self.event_repository
            .create_event(&input.account_id, CreateEventInput {
                board_id: input.board_id,
                eventable_id: card.id.clone(),
                eventable_type: "Card".to_string(),
                creator_id: input.user_id,
                action: event_actions::CARD_CREATED.to_string(),
                particulars: serde_json::json!({}),
            })
            .await;

        Ok(card)
    }
}
