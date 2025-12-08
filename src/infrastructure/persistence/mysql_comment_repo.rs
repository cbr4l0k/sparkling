use crate::domain::entities::Comment;
use crate::domain::errors::DomainError;
use crate::domain::ports::CommentRepository;
use crate::domain::value_objects::FizzyId;
use async_trait::async_trait;
use sqlx::MySqlPool;

pub struct MysqlCommentRepository {
    pool: MySqlPool,
}

impl MysqlCommentRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentRepository for MysqlCommentRepository {
    async fn list_for_card(
        &self,
        _account_id: &FizzyId,
        _card_id: &FizzyId,
        _limit: Option<i64>,
    ) -> Result<Vec<Comment>, DomainError> {
        // TODO: Implement in Phase 2
        todo!("list_for_card implementation")
    }

    async fn create(
        &self,
        _account_id: &FizzyId,
        _card_id: &FizzyId,
        _creator_id: &FizzyId,
        _content: &str,
    ) -> Result<Comment, DomainError> {
        // TODO: Implement in Phase 3
        todo!("create implementation")
    }
}
