use async_trait::async_trait;
use sqlx::MySqlPool;
use crate::domain::ports::{EventRepository, CreateEventInput};
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

pub struct MysqlEventRepository {
    pool: MySqlPool,
}

impl MysqlEventRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for MysqlEventRepository {
    async fn create_event(
        &self,
        _account_id: &FizzyId,
        _input: CreateEventInput,
    ) -> Result<(), DomainError> {
        // TODO: Implement in Phase 3
        todo!("create_event implementation")
    }
}
