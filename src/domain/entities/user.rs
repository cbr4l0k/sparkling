use chrono::{DateTime, Utc};
use crate::domain::value_objects::FizzyId;

/// User role in an account
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Owner,
    Admin,
    Member,
    System,
}

impl UserRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(UserRole::Owner),
            "admin" => Some(UserRole::Admin),
            "member" => Some(UserRole::Member),
            "system" => Some(UserRole::System),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Owner => "owner",
            UserRole::Admin => "admin",
            UserRole::Member => "member",
            UserRole::System => "system",
        }
    }
}

/// User within an account
#[derive(Debug, Clone)]
pub struct User {
    pub id: FizzyId,
    pub account_id: FizzyId,
    pub identity_id: Option<FizzyId>,
    pub name: String,
    pub role: UserRole,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // From identity
    pub email: Option<String>,
}

impl User {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Owner | UserRole::Admin)
    }
}
