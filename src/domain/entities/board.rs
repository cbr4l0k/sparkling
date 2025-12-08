use chrono::{DateTime, Utc};
use crate::domain::value_objects::FizzyId;

/// Organizational unit for cards (like a project)
#[derive(Debug, Clone)]
pub struct Board {
    pub id: FizzyId,
    pub account_id: FizzyId,
    pub creator_id: FizzyId,
    pub name: String,
    pub all_access: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Denormalized fields
    pub card_count: Option<i64>,
}

impl Board {
    /// Check if this board is accessible to all users in the account
    pub fn is_public(&self) -> bool {
        self.all_access
    }
}
