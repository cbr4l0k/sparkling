use async_trait::async_trait;
use sqlx::MySqlPool;
use crate::domain::entities::{Board, Column};
use crate::domain::ports::BoardRepository;
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

pub struct MysqlBoardRepository {
    pool: MySqlPool,
}

impl MysqlBoardRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardRepository for MysqlBoardRepository {
    async fn find_by_id(
        &self,
        _account_id: &FizzyId,
        _id: &FizzyId,
    ) -> Result<Option<Board>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("find_by_id implementation")
    }

    async fn find_by_name(
        &self,
        _account_id: &FizzyId,
        _name: &str,
    ) -> Result<Option<Board>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("find_by_name implementation")
    }

    async fn list_accessible(
        &self,
        _account_id: &FizzyId,
        _user_id: &FizzyId,
    ) -> Result<Vec<Board>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("list_accessible implementation")
    }

    async fn get_columns(
        &self,
        _account_id: &FizzyId,
        _board_id: &FizzyId,
    ) -> Result<Vec<Column>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("get_columns implementation")
    }

    async fn user_has_access(
        &self,
        _account_id: &FizzyId,
        _board_id: &FizzyId,
        _user_id: &FizzyId,
    ) -> Result<bool, DomainError> {
        // TODO: Implement in Phase 2
        todo!("user_has_access implementation")
    }
}
