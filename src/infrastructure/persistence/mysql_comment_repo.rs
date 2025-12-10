use crate::domain::entities::Comment;
use crate::domain::errors::DomainError;
use crate::domain::ports::CommentRepository;
use crate::domain::value_objects::FizzyId;
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};

pub struct SqliteCommentRepository {
    pool: SqlitePool,
}

impl SqliteCommentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentRepository for SqliteCommentRepository {
    async fn list_for_card(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
        limit: Option<i64>,
    ) -> Result<Vec<Comment>, DomainError> {
        let query = "
            SELECT
                c.id,
                c.account_id,
                c.card_id,
                c.creator_id,
                c.created_at,
                c.updated_at,
                rt. body as content
            FROM comments c
            INNER JOIN action_text_rich_texts rt
                ON c.id = rt.record_id
            WHERE c.account_id = ?
                AND c.card_id = ?
                AND c.record_type = 'Comment'
                AND rt.name = 'body'
            ORDER BY c.created_at DESC
            LIMIT ?
           ";
        let limit = limit.unwrap_or(50);

        let rows = sqlx::query(query)
            .bind(account_id)
            .bind(card_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let comments = rows
            .into_iter()
            .map(|row| {
                Ok(Comment {
                    id: row.try_get("id")?,
                    account_id: row.try_get("account_id")?,
                    creator_id: row.try_get("creator_id")?,
                    card_id: row.try_get("card_id")?,
                    content: row.try_get("content")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,

                    // TODO: Implement in phase 3
                    // For now user is None, because I don't want to currently
                    // deal with the users JOIN operation
                    creator_name: None,
                })
            })
            .collect::<Result<Vec<Comment>, sqlx::Error>>()
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(comments)
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
