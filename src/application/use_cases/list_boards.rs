use std::sync::Arc;
use crate::domain::entities::Board;
use crate::domain::ports::BoardRepository;
use crate::domain::value_objects::FizzyId;
use crate::application::errors::ApplicationError;

pub struct ListBoardsUseCase {
    board_repository: Arc<dyn BoardRepository>,
}

pub struct ListBoardsInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
}

pub struct ListBoardsOutput {
    pub boards: Vec<Board>,
}

impl ListBoardsUseCase {
    pub fn new(board_repository: Arc<dyn BoardRepository>) -> Self {
        Self { board_repository }
    }

    pub async fn execute(&self, input: ListBoardsInput) -> Result<ListBoardsOutput, ApplicationError> {
        let boards = self.board_repository
            .list_accessible(&input.account_id, &input.user_id)
            .await
            .map_err(ApplicationError::DomainError)?;

        Ok(ListBoardsOutput { boards })
    }
}
