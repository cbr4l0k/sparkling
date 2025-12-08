use std::sync::Arc;
use crate::domain::entities::Card;
use crate::domain::ports::{CardRepository, CardFilters, BoardRepository};
use crate::domain::value_objects::{FizzyId, CardStatus};
use crate::application::errors::ApplicationError;

pub struct ListBoardCardsUseCase {
    card_repository: Arc<dyn CardRepository>,
    board_repository: Arc<dyn BoardRepository>,
}

pub struct ListBoardCardsInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub board_name: String,
    pub limit: Option<i64>,
}

pub struct ListBoardCardsOutput {
    pub cards: Vec<Card>,
    pub board_name: String,
}

impl ListBoardCardsUseCase {
    pub fn new(
        card_repository: Arc<dyn CardRepository>,
        board_repository: Arc<dyn BoardRepository>,
    ) -> Self {
        Self { card_repository, board_repository }
    }

    pub async fn execute(&self, input: ListBoardCardsInput) -> Result<ListBoardCardsOutput, ApplicationError> {
        let board = self.board_repository
            .find_by_name(&input.account_id, &input.board_name)
            .await
            .map_err(ApplicationError::DomainError)?
            .ok_or_else(|| ApplicationError::NotFound(format!("Board '{}' not found", input.board_name)))?;

        let has_access = self.board_repository
            .user_has_access(&input.account_id, &board.id, &input.user_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        if !has_access {
            return Err(ApplicationError::Unauthorized("No access to this board".to_string()));
        }

        let filters = CardFilters {
            board_id: Some(board.id.clone()),
            exclude_status: Some(vec![CardStatus::Closed, CardStatus::NotNow]),
            limit: input.limit.or(Some(20)),
            ..Default::default()
        };

        let cards = self.card_repository
            .list(&input.account_id, filters)
            .await
            .map_err(ApplicationError::DomainError)?;

        Ok(ListBoardCardsOutput {
            cards,
            board_name: board.name,
        })
    }
}
