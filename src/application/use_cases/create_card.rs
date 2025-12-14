use crate::application::errors::ApplicationError;
use crate::domain::entities::Card;
use crate::domain::ports::{
    event_actions, BoardRepository, CardRepository, CreateCardInput, CreateEventInput,
    EventRepository,
};
use crate::domain::value_objects::{CardStatus, FizzyId};
use std::sync::Arc;

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
        Self {
            card_repository,
            board_repository,
            event_repository,
        }
    }

    pub async fn execute(&self, input: CreateCardUseCaseInput) -> Result<Card, ApplicationError> {
        // Verify access
        let has_access = self
            .board_repository
            .user_has_access(&input.account_id, &input.board_id, &input.user_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        if !has_access {
            return Err(ApplicationError::Unauthorized(
                "No access to this board".to_string(),
            ));
        }

        // Get the first column of the board to place the card directly on the board
        let columns = self
            .board_repository
            .get_columns(&input.account_id, &input.board_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        let first_column_id = columns.first().map(|c| c.id.clone());

        // If there's a column, create as published (visible on board), otherwise drafted
        let (status, column_id) = if first_column_id.is_some() {
            (CardStatus::Published, first_column_id)
        } else {
            (CardStatus::Drafted, None)
        };

        let create_input = CreateCardInput {
            board_id: input.board_id.clone(),
            creator_id: input.user_id.clone(),
            title: input.title,
            description: input.description,
            status,
            column_id,
        };

        let card = self
            .card_repository
            .create(&input.account_id, create_input)
            .await
            .map_err(ApplicationError::DomainError)?;

        // Create event for audit trail
        let _ = self
            .event_repository
            .create_event(
                &input.account_id,
                CreateEventInput {
                    board_id: input.board_id,
                    eventable_id: card.id.clone(),
                    eventable_type: "Card".to_string(),
                    creator_id: input.user_id,
                    action: event_actions::CARD_CREATED.to_string(),
                    particulars: serde_json::json!({}),
                },
            )
            .await;

        Ok(card)
    }
}
