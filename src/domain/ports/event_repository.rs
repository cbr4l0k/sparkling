use crate::domain::errors::DomainError;
use crate::domain::value_objects::FizzyId;
use async_trait::async_trait;

/// Input for creating an event (audit trail)
#[derive(Debug, Clone)]
pub struct CreateEventInput {
    pub board_id: FizzyId,
    pub eventable_id: FizzyId,
    pub eventable_type: String,
    pub creator_id: FizzyId,
    pub action: String,
    pub particulars: serde_json::Value,
}

/// Common event actions used by Fizzy
pub mod event_actions {
    pub const CARD_CREATED: &str = "card_created";
    pub const CARD_UPDATED: &str = "card_updated";
    pub const CARD_CLOSED: &str = "card_closed";
    pub const CARD_REOPENED: &str = "card_reopened";
    pub const CARD_COLUMN_CHANGED: &str = "card_column_changed";
    pub const CARD_BOARD_CHANGED: &str = "card_board_changed";
    pub const COMMENT_CREATED: &str = "comment_created";
}

/// Port for event repository operations (audit trail)
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Create an event record for the activity timeline
    async fn create_event(
        &self,
        account_id: &FizzyId,
        input: CreateEventInput,
    ) -> Result<(), DomainError>;
}
