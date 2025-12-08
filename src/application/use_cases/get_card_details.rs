use std::sync::Arc;
use crate::domain::entities::Card;
use crate::domain::ports::CardRepository;
use crate::domain::value_objects::FizzyId;
use crate::application::errors::ApplicationError;

pub struct GetCardDetailsUseCase {
    card_repository: Arc<dyn CardRepository>,
}

pub struct GetCardDetailsInput {
    pub account_id: FizzyId,
    pub card_number: i64,
}

impl GetCardDetailsUseCase {
    pub fn new(card_repository: Arc<dyn CardRepository>) -> Self {
        Self { card_repository }
    }

    pub async fn execute(&self, input: GetCardDetailsInput) -> Result<Card, ApplicationError> {
        self.card_repository
            .find_by_number(&input.account_id, input.card_number)
            .await
            .map_err(ApplicationError::DomainError)?
            .ok_or_else(|| ApplicationError::NotFound(format!("Card #{} not found", input.card_number)))
    }
}
