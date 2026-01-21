use crate::application::errors::ApplicationError;
use crate::domain::entities::Card;
use crate::domain::ports::{CardFilters, CardRepository};
use crate::domain::value_objects::FizzyId;
use std::sync::Arc;

pub struct ListMyCardsUseCase {
    card_repository: Arc<dyn CardRepository>,
}

#[derive(Debug)]
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

    pub async fn execute(
        &self,
        input: ListMyCardsInput,
    ) -> Result<ListMyCardsOutput, ApplicationError> {
        let filters = CardFilters {
            creator_id: Some(input.user_id),
            exclude_closed: if input.include_closed {
                None
            } else {
                Some(true)
            },
            limit: input.limit.or(Some(20)),
            ..Default::default()
        };

        let cards = self
            .card_repository
            .list(&input.account_id, filters)
            .await
            .map_err(ApplicationError::DomainError)?;

        Ok(ListMyCardsOutput { cards })
    }
}
