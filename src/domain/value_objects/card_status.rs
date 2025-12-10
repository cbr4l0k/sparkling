use std::{fmt, str::FromStr};

/// Card status in the Fizzy workflow.
/// - Drafted: Initial state, in triage area
/// - Triaged: Published to a column, actively being worked on
/// - Closed: Completed/resolved
/// - NotNow: Postponed for later
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CardStatus {
    #[default]
    Drafted,
    Triaged,
    Closed,
    NotNow,
    Published,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CardStatusError;

impl fmt::Display for CardStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid card status")
    }
}

impl FromStr for CardStatus {
    type Err = CardStatusError;

    /// Parse from database string representation
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "drafted" => Ok(CardStatus::Drafted),
            "published" => Ok(CardStatus::Published),
            "triaged" => Ok(CardStatus::Triaged),
            "closed" => Ok(CardStatus::Closed),
            "not_now" => Ok(CardStatus::NotNow),
            _ => Err(CardStatusError),
        }
    }
}

impl CardStatus {
    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            CardStatus::Drafted => "drafted",
            CardStatus::Triaged => "triaged",
            CardStatus::Closed => "closed",
            CardStatus::NotNow => "not_now",
            CardStatus::Published => "published",
        }
    }

    /// Check if this status represents an active card
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            CardStatus::Drafted | CardStatus::Triaged | CardStatus::Published
        )
    }

    /// Get emoji representation for Telegram display
    pub fn emoji(&self) -> &'static str {
        match self {
            CardStatus::Drafted => "📝",
            CardStatus::Triaged => "📋",
            CardStatus::Closed => "✅",
            CardStatus::NotNow => "⏸️",
            CardStatus::Published => "📢",
        }
    }

    /// Get human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CardStatus::Drafted => "Draft",
            CardStatus::Triaged => "Active",
            CardStatus::Closed => "Closed",
            CardStatus::NotNow => "Postponed",
            CardStatus::Published => "Published",
        }
    }
}

impl fmt::Display for CardStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        for status in [
            CardStatus::Drafted,
            CardStatus::Triaged,
            CardStatus::Closed,
            CardStatus::NotNow,
            CardStatus::Published,
        ] {
            let s = status.as_str();
            let parsed = CardStatus::from_str(s);
            assert_eq!(parsed, Ok(status));
        }
    }

    #[test]
    fn test_is_active() {
        assert!(CardStatus::Drafted.is_active());
        assert!(CardStatus::Triaged.is_active());
        assert!(CardStatus::Published.is_active());
        assert!(!CardStatus::Closed.is_active());
        assert!(!CardStatus::NotNow.is_active());
    }
}
