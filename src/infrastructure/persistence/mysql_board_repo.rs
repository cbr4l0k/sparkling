use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use crate::domain::entities::{Board, Column};
use crate::domain::ports::BoardRepository;
use crate::domain::value_objects::FizzyId;
use crate::domain::errors::DomainError;

pub struct SqliteBoardRepository {
    pool: SqlitePool,
}

/// Raw row from database query for boards
#[derive(Debug, FromRow)]
struct BoardRow {
    id: FizzyId,
    account_id: FizzyId,
    creator_id: FizzyId,
    name: String,
    all_access: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    card_count: Option<i64>,
}

impl From<BoardRow> for Board {
    fn from(row: BoardRow) -> Self {
        Board {
            id: row.id,
            account_id: row.account_id,
            creator_id: row.creator_id,
            name: row.name,
            all_access: row.all_access,
            created_at: row.created_at,
            updated_at: row.updated_at,
            card_count: row.card_count,
        }
    }
}

/// Raw row from database query for columns
#[derive(Debug, FromRow)]
struct ColumnRow {
    id: FizzyId,
    account_id: FizzyId,
    board_id: FizzyId,
    name: String,
    color: String,
    position: i32,
}

impl From<ColumnRow> for Column {
    fn from(row: ColumnRow) -> Self {
        Column {
            id: row.id,
            account_id: row.account_id,
            board_id: row.board_id,
            name: row.name,
            color: row.color,
            position: row.position,
        }
    }
}

impl SqliteBoardRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardRepository for SqliteBoardRepository {
    async fn find_by_id(
        &self,
        account_id: &FizzyId,
        id: &FizzyId,
    ) -> Result<Option<Board>, DomainError> {
        let row = sqlx::query_as::<_, BoardRow>(
            r#"
            SELECT
                b.id,
                b.account_id,
                b.creator_id,
                b.name,
                b.all_access,
                b.created_at,
                b.updated_at,
                (SELECT COUNT(*) FROM cards c WHERE c.board_id = b.id AND c.status NOT IN ('closed', 'not_now')) as card_count
            FROM boards b
            WHERE b.account_id = ? AND b.id = ?
            "#
        )
        .bind(account_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(Board::from))
    }

    async fn find_by_name(
        &self,
        account_id: &FizzyId,
        name: &str,
    ) -> Result<Option<Board>, DomainError> {
        let row = sqlx::query_as::<_, BoardRow>(
            r#"
            SELECT
                b.id,
                b.account_id,
                b.creator_id,
                b.name,
                b.all_access,
                b.created_at,
                b.updated_at,
                (SELECT COUNT(*) FROM cards c WHERE c.board_id = b.id AND c.status NOT IN ('closed', 'not_now')) as card_count
            FROM boards b
            WHERE b.account_id = ? AND LOWER(b.name) = LOWER(?)
            "#
        )
        .bind(account_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(Board::from))
    }

    async fn list_accessible(
        &self,
        account_id: &FizzyId,
        user_id: &FizzyId,
    ) -> Result<Vec<Board>, DomainError> {
        // A user can access a board if:
        // 1. The board has all_access = true, OR
        // 2. The user has an access record for the board
        let rows = sqlx::query_as::<_, BoardRow>(
            r#"
            SELECT DISTINCT
                b.id,
                b.account_id,
                b.creator_id,
                b.name,
                b.all_access,
                b.created_at,
                b.updated_at,
                (SELECT COUNT(*) FROM cards c WHERE c.board_id = b.id AND c.status NOT IN ('closed', 'not_now')) as card_count
            FROM boards b
            LEFT JOIN accesses a ON b.id = a.board_id AND a.user_id = ?
            WHERE b.account_id = ?
              AND (b.all_access = true OR a.id IS NOT NULL)
            ORDER BY b.name ASC
            "#
        )
        .bind(user_id)
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(Board::from).collect())
    }

    async fn get_columns(
        &self,
        account_id: &FizzyId,
        board_id: &FizzyId,
    ) -> Result<Vec<Column>, DomainError> {
        let rows = sqlx::query_as::<_, ColumnRow>(
            r#"
            SELECT
                id,
                account_id,
                board_id,
                name,
                color,
                position
            FROM columns
            WHERE account_id = ? AND board_id = ?
            ORDER BY position ASC
            "#
        )
        .bind(account_id)
        .bind(board_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(Column::from).collect())
    }

    async fn user_has_access(
        &self,
        account_id: &FizzyId,
        board_id: &FizzyId,
        user_id: &FizzyId,
    ) -> Result<bool, DomainError> {
        // Check if board exists and if user has access
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT CASE
                WHEN b.all_access = true THEN true
                WHEN a.id IS NOT NULL THEN true
                ELSE false
            END as has_access
            FROM boards b
            LEFT JOIN accesses a ON b.id = a.board_id AND a.user_id = ?
            WHERE b.account_id = ? AND b.id = ?
            "#
        )
        .bind(user_id)
        .bind(account_id)
        .bind(board_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(result.unwrap_or(false))
    }
}
