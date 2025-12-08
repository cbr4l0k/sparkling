use chrono::{DateTime, NaiveDate, Utc};
use crate::domain::value_objects::{FizzyId, CardStatus};

/// Main work item (task/issue) in Fizzy
#[derive(Debug, Clone)]
pub struct Card {
    pub id: FizzyId,
    pub account_id: FizzyId,
    pub board_id: FizzyId,
    pub column_id: Option<FizzyId>,
    pub creator_id: FizzyId,
    pub number: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: CardStatus,
    pub due_on: Option<NaiveDate>,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Denormalized fields for display (loaded via JOINs)
    pub board_name: Option<String>,
    pub column_name: Option<String>,
    pub column_color: Option<String>,
    pub creator_name: Option<String>,
    pub assignee_names: Vec<String>,
    pub tag_titles: Vec<String>,
    pub is_golden: bool,
}

impl Card {
    /// Format card number with # prefix for display
    pub fn formatted_number(&self) -> String {
        format!("#{}", self.number)
    }

    /// Check if the card is in an active state
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Generate a URL to view the card in Fizzy web UI
    pub fn web_url(&self, base_url: Option<&str>) -> Option<String> {
        base_url.map(|url| {
            format!("{}/{}/cards/{}", url.trim_end_matches('/'), self.account_id, self.number)
        })
    }
}
