use std::sync::Arc;
use crate::domain::entities::Card;
use crate::domain::ports::{CardRepository, UpdateCardInput, BoardRepository, EventRepository, CreateEventInput, event_actions};
use crate::domain::value_objects::{FizzyId, CardStatus};
use crate::application::errors::ApplicationError;

pub struct MoveCardUseCase {
    card_repository: Arc<dyn CardRepository>,
    board_repository: Arc<dyn BoardRepository>,
    event_repository: Arc<dyn EventRepository>,
}

pub struct MoveCardInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub card_number: i64,
    pub column_id: FizzyId,
}

impl MoveCardUseCase {
    pub fn new(
        card_repository: Arc<dyn CardRepository>,
        board_repository: Arc<dyn BoardRepository>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self { card_repository, board_repository, event_repository }
    }

    pub async fn execute(&self, input: MoveCardInput) -> Result<Card, ApplicationError> {
        let card = self.card_repository
            .find_by_number(&input.account_id, input.card_number)
            .await
            .map_err(ApplicationError::DomainError)?
            .ok_or_else(|| ApplicationError::NotFound(format!("Card #{} not found", input.card_number)))?;

        // Verify column belongs to the board
        let columns = self.board_repository
            .get_columns(&input.account_id, &card.board_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        if !columns.iter().any(|c| c.id == input.column_id) {
            return Err(ApplicationError::InvalidInput("Column not in this board".to_string()));
        }

        let update_input = UpdateCardInput {
            column_id: Some(input.column_id.clone()),
            status: Some(CardStatus::Triaged),
            ..Default::default()
        };

        let updated = self.card_repository
            .update(&input.account_id, &card.id, update_input)
            .await
            .map_err(ApplicationError::DomainError)?;

        // Create event
        let _ = self.event_repository
            .create_event(&input.account_id, CreateEventInput {
                board_id: card.board_id.clone(),
                eventable_id: card.id.clone(),
                eventable_type: "Card".to_string(),
                creator_id: input.user_id,
                action: event_actions::CARD_COLUMN_CHANGED.to_string(),
                particulars: serde_json::json!({
                    "column_id": input.column_id.as_str()
                }),
            })
            .await;

        Ok(updated)
    }
}
