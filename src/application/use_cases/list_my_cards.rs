use std::sync::Arc;
use crate::domain::entities::Card;
use crate::domain::ports::{CardRepository, CardFilters};
use crate::domain::value_objects::{FizzyId, CardStatus};
use crate::application::errors::ApplicationError;

pub struct ListMyCardsUseCase {
    card_repository: Arc<dyn CardRepository>,
}

pub struct ListMyCardsInput {
    pub account_id: FizzyId,
    pub user_id: FizzyId,
    pub include_closed: bool,
    pub limit: Option<i64>,
}

pub struct ListMyCardsOutput {
    pub cards: Vec<Card>,
}

impl ListMyCardsUseCase {
    pub fn new(card_repository: Arc<dyn CardRepository>) -> Self {
        Self { card_repository }
    }

    pub async fn execute(&self, input: ListMyCardsInput) -> Result<ListMyCardsOutput, ApplicationError> {
        let filters = CardFilters {
            assignee_id: Some(input.user_id),
            exclude_status: if input.include_closed {
                None
            } else {
                Some(vec![CardStatus::Closed, CardStatus::NotNow])
            },
            limit: input.limit.or(Some(20)),
            ..Default::default()
        };

        let cards = self.card_repository
            .list(&input.account_id, filters)
            .await
            .map_err(ApplicationError::DomainError)?;

        Ok(ListMyCardsOutput { cards })
    }
}
