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
        account_id: &FizzyId,
        input: CreateEventInput,
    ) -> Result<(), DomainError> {
        let event_id = FizzyId::generate();
        let particulars_json = input.particulars.to_string();

        sqlx::query(
            r#"
            INSERT INTO events (
                id, account_id, board_id, eventable_id, eventable_type,
                creator_id, action, particulars, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
        )
        .bind(&event_id)
        .bind(account_id)
        .bind(&input.board_id)
        .bind(&input.eventable_id)
        .bind(&input.eventable_type)
        .bind(&input.creator_id)
        .bind(&input.action)
        .bind(&particulars_json)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
