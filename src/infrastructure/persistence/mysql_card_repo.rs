use async_trait::async_trait;
use sqlx::MySqlPool;
use crate::domain::entities::Card;
use crate::domain::ports::{CardRepository, CardFilters, CreateCardInput, UpdateCardInput};
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

pub struct MysqlCardRepository {
    pool: MySqlPool,
}

impl MysqlCardRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CardRepository for MysqlCardRepository {
    async fn find_by_number(
        &self,
        _account_id: &FizzyId,
        _number: i64,
    ) -> Result<Option<Card>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("find_by_number implementation")
    }

    async fn find_by_id(
        &self,
        _account_id: &FizzyId,
        _id: &FizzyId,
    ) -> Result<Option<Card>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("find_by_id implementation")
    }

    async fn list(
        &self,
        _account_id: &FizzyId,
        _filters: CardFilters,
    ) -> Result<Vec<Card>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("list implementation")
    }

    async fn create(
        &self,
        _account_id: &FizzyId,
        _input: CreateCardInput,
    ) -> Result<Card, DomainError> {
        // TODO: Implement in Phase 3
        todo!("create implementation")
    }

    async fn update(
        &self,
        _account_id: &FizzyId,
        _card_id: &FizzyId,
        _input: UpdateCardInput,
    ) -> Result<Card, DomainError> {
        // TODO: Implement in Phase 3
        todo!("update implementation")
    }

    async fn close(
        &self,
        _account_id: &FizzyId,
        _card_id: &FizzyId,
        _user_id: &FizzyId,
    ) -> Result<(), DomainError> {
        // TODO: Implement in Phase 3
        todo!("close implementation")
    }

    async fn reopen(
        &self,
        _account_id: &FizzyId,
        _card_id: &FizzyId,
    ) -> Result<(), DomainError> {
        // TODO: Implement in Phase 3
        todo!("reopen implementation")
    }
}
