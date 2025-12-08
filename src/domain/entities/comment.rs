use crate::domain::value_objects::FizzyId;
use chrono::{DateTime, Utc};

/// Comment on a card
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: FizzyId,
    pub account_id: FizzyId,
    pub card_id: FizzyId,
    pub creator_id: FizzyId,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Denormalized fields
    pub creator_name: Option<String>,
}
