use crate::domain::entities::Card;
use crate::domain::errors::DomainError;
use crate::domain::ports::{CardFilters, CardRepository, CreateCardInput, UpdateCardInput};
use crate::domain::value_objects::{CardStatus, FizzyId};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool};

pub struct SqliteCardRepository {
    pool: SqlitePool,
}

/// Raw row from the database query
#[derive(Debug, FromRow)]
struct CardRow {
    id: FizzyId,
    account_id: FizzyId,
    board_id: FizzyId,
    column_id: Option<FizzyId>,
    creator_id: FizzyId,
    number: i64,
    title: String,
    status: String,
    due_on: Option<NaiveDate>,
    last_active_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    board_name: Option<String>,
    column_name: Option<String>,
    column_color: Option<String>,
    creator_name: Option<String>,
    is_golden: bool,
}

impl CardRow {
    fn into_card(
        self,
        assignee_names: Vec<String>,
        tag_titles: Vec<String>,
    ) -> Result<Card, DomainError> {
        let status = self
            .status
            .parse::<CardStatus>()
            .map_err(|_| DomainError::InvalidState {
                message: format!("Invalid card status: {}", self.status),
            })?;

        Ok(Card {
            id: self.id,
            account_id: self.account_id,
            board_id: self.board_id,
            column_id: self.column_id,
            creator_id: self.creator_id,
            number: self.number,
            title: self.title,
            description: None, // Rich text loaded separately if needed
            status,
            due_on: self.due_on,
            last_active_at: self.last_active_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            board_name: self.board_name,
            column_name: self.column_name,
            column_color: self.column_color,
            creator_name: self.creator_name,
            assignee_names,
            tag_titles,
            is_golden: self.is_golden,
        })
    }
}

impl SqliteCardRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Load assignee names for a card
    async fn load_assignees(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
    ) -> Result<Vec<String>, DomainError> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT u.name
            FROM assignments a
            JOIN users u ON a.assignee_id = u.id
            WHERE a.account_id = ? AND a.card_id = ?
            "#,
        )
        .bind(account_id)
        .bind(card_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows)
    }

    /// Load tag titles for a card
    async fn load_tags(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
    ) -> Result<Vec<String>, DomainError> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT t.title
            FROM taggings tg
            JOIN tags t ON tg.tag_id = t.id
            WHERE tg.account_id = ? AND tg.card_id = ?
            "#,
        )
        .bind(account_id)
        .bind(card_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows)
    }

    /// Base query for loading cards with all JOINs
    fn base_card_query() -> &'static str {
        r#"
        SELECT
            c.id,
            c.account_id,
            c.board_id,
            c.column_id,
            c.creator_id,
            c.number,
            c.title,
            c.status,
            c.due_on,
            c.last_active_at,
            c.created_at,
            c.updated_at,
            b.name as board_name,
            col.name as column_name,
            col.color as column_color,
            u.name as creator_name,
            CASE WHEN cg.id IS NOT NULL THEN true ELSE false END as is_golden
        FROM cards c
        JOIN boards b ON c.board_id = b.id
        LEFT JOIN columns col ON c.column_id = col.id
        JOIN users u ON c.creator_id = u.id
        LEFT JOIN card_goldnesses cg ON c.id = cg.card_id
        "#
    }
}

#[async_trait]
impl CardRepository for SqliteCardRepository {
    async fn find_by_number(
        &self,
        account_id: &FizzyId,
        number: i64,
    ) -> Result<Option<Card>, DomainError> {
        let query = format!(
            "{} WHERE c.account_id = ? AND c.number = ?",
            Self::base_card_query()
        );

        let row = sqlx::query_as::<_, CardRow>(&query)
            .bind(account_id)
            .bind(number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        match row {
            Some(card_row) => {
                let assignees = self.load_assignees(account_id, &card_row.id).await?;
                let tags = self.load_tags(account_id, &card_row.id).await?;
                Ok(Some(card_row.into_card(assignees, tags)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_id(
        &self,
        account_id: &FizzyId,
        id: &FizzyId,
    ) -> Result<Option<Card>, DomainError> {
        let query = format!(
            "{} WHERE c.account_id = ? AND c.id = ?",
            Self::base_card_query()
        );

        let row = sqlx::query_as::<_, CardRow>(&query)
            .bind(account_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        match row {
            Some(card_row) => {
                let assignees = self.load_assignees(account_id, &card_row.id).await?;
                let tags = self.load_tags(account_id, &card_row.id).await?;
                Ok(Some(card_row.into_card(assignees, tags)?))
            }
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        account_id: &FizzyId,
        filters: CardFilters,
    ) -> Result<Vec<Card>, DomainError> {
        let mut query = format!("{} WHERE c.account_id = ?", Self::base_card_query());
        let mut bindings: Vec<String> = vec![account_id.to_string()];

        // Add assignee filter (requires JOIN)
        if let Some(ref assignee_id) = filters.assignee_id {
            query.push_str(" AND EXISTS (SELECT 1 FROM assignments a WHERE a.card_id = c.id AND a.assignee_id = ?)");
            bindings.push(assignee_id.to_string());
        }

        // Add board filter
        if let Some(ref board_id) = filters.board_id {
            query.push_str(" AND c.board_id = ?");
            bindings.push(board_id.to_string());
        }

        // Add column filter
        if let Some(ref column_id) = filters.column_id {
            query.push_str(" AND c.column_id = ?");
            bindings.push(column_id.to_string());
        }

        // Add status inclusion filter
        if let Some(ref statuses) = filters.status {
            if !statuses.is_empty() {
                let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                query.push_str(&format!(" AND c.status IN ({})", placeholders));
                for status in statuses {
                    bindings.push(status.as_str().to_string());
                }
            }
        }

        // Add status exclusion filter
        if let Some(ref exclude_statuses) = filters.exclude_status {
            if !exclude_statuses.is_empty() {
                let placeholders = exclude_statuses
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(", ");
                query.push_str(&format!(" AND c.status NOT IN ({})", placeholders));
                for status in exclude_statuses {
                    bindings.push(status.as_str().to_string());
                }
            }
        }

        // Add golden filter
        if let Some(is_golden) = filters.is_golden {
            if is_golden {
                query.push_str(" AND cg.id IS NOT NULL");
            } else {
                query.push_str(" AND cg.id IS NULL");
            }
        }

        // Order by last_active_at descending
        query.push_str(" ORDER BY c.last_active_at DESC");

        // Add limit
        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // Add offset
        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Build the query with dynamic bindings
        // SQLx doesn't support dynamic binding easily, so we rebuild with proper types
        let rows = self
            .execute_list_query(&query, account_id, &filters)
            .await?;

        // Load assignees and tags for each card
        let mut cards = Vec::with_capacity(rows.len());
        for row in rows {
            let assignees = self.load_assignees(account_id, &row.id).await?;
            let tags = self.load_tags(account_id, &row.id).await?;
            cards.push(row.into_card(assignees, tags)?);
        }

        Ok(cards)
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

    async fn reopen(&self, _account_id: &FizzyId, _card_id: &FizzyId) -> Result<(), DomainError> {
        // TODO: Implement in Phase 3
        todo!("reopen implementation")
    }
}

impl SqliteCardRepository {
    /// Execute list query with proper bindings
    /// This is separated because SQLx requires compile-time known bindings
    async fn execute_list_query(
        &self,
        _query_template: &str,
        account_id: &FizzyId,
        filters: &CardFilters,
    ) -> Result<Vec<CardRow>, DomainError> {
        // Build dynamic query with all conditions
        let mut conditions = vec!["c.account_id = ?".to_string()];

        if filters.assignee_id.is_some() {
            conditions.push(
                "EXISTS (SELECT 1 FROM assignments a WHERE a.card_id = c.id AND a.assignee_id = ?)"
                    .to_string(),
            );
        }

        if filters.board_id.is_some() {
            conditions.push("c.board_id = ?".to_string());
        }

        if filters.column_id.is_some() {
            conditions.push("c.column_id = ?".to_string());
        }

        if let Some(ref statuses) = filters.status {
            if !statuses.is_empty() {
                let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                conditions.push(format!("c.status IN ({})", placeholders));
            }
        }

        if let Some(ref exclude_statuses) = filters.exclude_status {
            if !exclude_statuses.is_empty() {
                let placeholders = exclude_statuses
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("c.status NOT IN ({})", placeholders));
            }
        }

        if let Some(is_golden) = filters.is_golden {
            if is_golden {
                conditions.push("cg.id IS NOT NULL".to_string());
            } else {
                conditions.push("cg.id IS NULL".to_string());
            }
        }

        let where_clause = conditions.join(" AND ");
        let mut query = format!(
            r#"
            SELECT
                c.id,
                c.account_id,
                c.board_id,
                c.column_id,
                c.creator_id,
                c.number,
                c.title,
                c.status,
                c.due_on,
                c.last_active_at,
                c.created_at,
                c.updated_at,
                b.name as board_name,
                col.name as column_name,
                col.color as column_color,
                u.name as creator_name,
                CASE WHEN cg.id IS NOT NULL THEN true ELSE false END as is_golden
            FROM cards c
            JOIN boards b ON c.board_id = b.id
            LEFT JOIN columns col ON c.column_id = col.id
            JOIN users u ON c.creator_id = u.id
            LEFT JOIN card_goldnesses cg ON c.id = cg.card_id
            WHERE {}
            ORDER BY c.last_active_at DESC
            "#,
            where_clause
        );

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        // Now bind values in order
        let mut query_builder = sqlx::query_as::<_, CardRow>(&query);

        // Always bind account_id first
        query_builder = query_builder.bind(account_id);

        // Bind optional filters in order
        if let Some(ref assignee_id) = filters.assignee_id {
            query_builder = query_builder.bind(assignee_id);
        }

        if let Some(ref board_id) = filters.board_id {
            query_builder = query_builder.bind(board_id);
        }

        if let Some(ref column_id) = filters.column_id {
            query_builder = query_builder.bind(column_id);
        }

        if let Some(ref statuses) = filters.status {
            for status in statuses {
                query_builder = query_builder.bind(status.as_str());
            }
        }

        if let Some(ref exclude_statuses) = filters.exclude_status {
            for status in exclude_statuses {
                query_builder = query_builder.bind(status.as_str());
            }
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows)
    }
}
