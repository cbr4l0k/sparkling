use crate::domain::entities::Card;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{CardStatus, FizzyId};
use async_trait::async_trait;
use chrono::NaiveDate;

/// Input for creating a new card
#[derive(Debug, Clone)]
pub struct CreateCardInput {
    pub board_id: FizzyId,
    pub creator_id: FizzyId,
    pub title: String,
    pub description: Option<String>,
    pub status: CardStatus,
    pub column_id: Option<FizzyId>,
}

/// Input for updating a card
#[derive(Debug, Clone, Default)]
pub struct UpdateCardInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<CardStatus>,
    pub column_id: Option<FizzyId>,
    pub due_on: Option<NaiveDate>,
}

/// Filters for listing cards
#[derive(Debug, Clone, Default)]
pub struct CardFilters {
    pub assignee_id: Option<FizzyId>,
    pub creator_id: Option<FizzyId>,
    pub board_id: Option<FizzyId>,
    pub column_id: Option<FizzyId>,
    pub status: Option<Vec<CardStatus>>,
    pub exclude_status: Option<Vec<CardStatus>>,
    pub is_golden: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Port for card repository operations
#[async_trait]
pub trait CardRepository: Send + Sync {
    /// Find a card by its account-scoped number
    async fn find_by_number(
        &self,
        account_id: &FizzyId,
        number: i64,
    ) -> Result<Option<Card>, DomainError>;

    /// Find a card by its ID
    async fn find_by_id(
        &self,
        account_id: &FizzyId,
        id: &FizzyId,
    ) -> Result<Option<Card>, DomainError>;

    /// List cards with filters
    async fn list(
        &self,
        account_id: &FizzyId,
        filters: CardFilters,
    ) -> Result<Vec<Card>, DomainError>;

    /// Create a new card
    async fn create(
        &self,
        account_id: &FizzyId,
        input: CreateCardInput,
    ) -> Result<Card, DomainError>;

    /// Update an existing card
    async fn update(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
        input: UpdateCardInput,
    ) -> Result<Card, DomainError>;

    /// Close a card
    async fn close(
        &self,
        account_id: &FizzyId,
        card_id: &FizzyId,
        user_id: &FizzyId,
    ) -> Result<(), DomainError>;

    /// Reopen a closed card
    async fn reopen(&self, account_id: &FizzyId, card_id: &FizzyId) -> Result<(), DomainError>;
}
