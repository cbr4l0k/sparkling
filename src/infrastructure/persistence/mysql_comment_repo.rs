use crate::domain::entities::Comment;
use crate::domain::errors::DomainError;
use crate::domain::ports::CommentRepository;
use crate::domain::value_objects::FizzyId;
use async_trait::async_trait;
use chrono::Utc;
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
        account_id: &FizzyId,
        card_id: &FizzyId,
        creator_id: &FizzyId,
        content: &str,
    ) -> Result<Comment, DomainError> {
        let comment_id = FizzyId::generate();
        let now = Utc::now();

        // Start a transaction
        let mut tx = self.pool.begin().await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        // 1. Insert comment record
        sqlx::query(
            r#"
            INSERT INTO comments (id, account_id, card_id, creator_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
            "#,
        )
        .bind(&comment_id)
        .bind(account_id)
        .bind(card_id)
        .bind(creator_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        // 2. Insert body into action_text_rich_texts
        let rich_text_id = FizzyId::generate();
        sqlx::query(
            r#"
            INSERT INTO action_text_rich_texts (
                id, account_id, record_type, record_id, name, body, created_at, updated_at
            )
            VALUES (?, ?, 'Comment', ?, 'body', ?, datetime('now'), datetime('now'))
            "#,
        )
        .bind(&rich_text_id)
        .bind(account_id)
        .bind(&comment_id)
        .bind(content)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        // 3. Update card's last_active_at
        sqlx::query(
            r#"
            UPDATE cards
            SET last_active_at = datetime('now'), updated_at = datetime('now')
            WHERE id = ? AND account_id = ?
            "#,
        )
        .bind(card_id)
        .bind(account_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        tx.commit().await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        // Return the created comment
        Ok(Comment {
            id: comment_id,
            account_id: account_id.clone(),
            card_id: card_id.clone(),
            creator_id: creator_id.clone(),
            content: content.to_string(),
            created_at: now,
            updated_at: now,
            creator_name: None,
        })
    }
}
