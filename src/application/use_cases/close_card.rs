use std::sync::Arc;
use crate::domain::ports::{CardRepository, EventRepository, CreateEventInput, event_actions};
use crate::domain::value_objects::FizzyId;
use crate::application::errors::ApplicationError;

pub struct CloseCardUseCase {
    card_repository: Arc<dyn CardRepository>,
    event_repository: Arc<dyn EventRepository>,
}

pub struct CloseCardInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub card_number: i64,
}

impl CloseCardUseCase {
    pub fn new(
        card_repository: Arc<dyn CardRepository>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self { card_repository, event_repository }
    }

    pub async fn execute(&self, input: CloseCardInput) -> Result<(), ApplicationError> {
        let card = self.card_repository
            .find_by_number(&input.account_id, input.card_number)
            .await
            .map_err(ApplicationError::DomainError)?
            .ok_or_else(|| ApplicationError::NotFound(format!("Card #{} not found", input.card_number)))?;

        self.card_repository
            .close(&input.account_id, &card.id, &input.user_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        // Create event
        let _ = self.event_repository
            .create_event(&input.account_id, CreateEventInput {
                board_id: card.board_id.clone(),
                eventable_id: card.id.clone(),
                eventable_type: "Card".to_string(),
                creator_id: input.user_id,
                action: event_actions::CARD_CLOSED.to_string(),
                particulars: serde_json::json!({}),
            })
            .await;

        Ok(())
    }
}
