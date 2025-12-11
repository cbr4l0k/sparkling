use crate::application::errors::ApplicationError;
use crate::domain::entities::Card;
use crate::domain::ports::{
    event_actions, CardRepository, CreateEventInput, EventRepository, UpdateCardInput,
};
use crate::domain::value_objects::FizzyId;
use std::sync::Arc;

pub struct UpdateCardUseCase {
    card_repository: Arc<dyn CardRepository>,
    event_repository: Arc<dyn EventRepository>,
}

pub struct UpdateCardUseCaseInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub card_number: i64,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl UpdateCardUseCase {
    pub fn new(
        card_repository: Arc<dyn CardRepository>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self {
            card_repository,
            event_repository,
        }
    }

    pub async fn execute(&self, input: UpdateCardUseCaseInput) -> Result<Card, ApplicationError> {
        let card = self
            .card_repository
            .find_by_number(&input.account_id, input.card_number)
            .await
            .map_err(ApplicationError::DomainError)?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!("Card #{} not found", input.card_number))
            })?;

        let update_input = UpdateCardInput {
            title: input.title,
            description: input.description,
            ..Default::default()
        };

        let updated = self
            .card_repository
            .update(&input.account_id, &card.id, update_input)
            .await
            .map_err(ApplicationError::DomainError)?;

        // Create event
        let _ = self
            .event_repository
            .create_event(
                &input.account_id,
                CreateEventInput {
                    board_id: card.board_id.clone(),
                    eventable_id: card.id.clone(),
                    eventable_type: "Card".to_string(),
                    creator_id: input.user_id,
                    action: event_actions::CARD_UPDATED.to_string(),
                    particulars: serde_json::json!({}),
                },
            )
            .await;

        Ok(updated)
    }
}
