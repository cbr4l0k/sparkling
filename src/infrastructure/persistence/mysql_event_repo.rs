use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::domain::ports::{EventRepository, CreateEventInput};
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

pub struct SqliteEventRepository {
    pool: SqlitePool,
}

impl SqliteEventRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for SqliteEventRepository {
    async fn create_event(
        &self,
        _account_id: &FizzyId,
        _input: CreateEventInput,
    ) -> Result<(), DomainError> {
        // TODO: Implement in Phase 3
        todo!("create_event implementation")
    }
}
