pub mod board_repository;
pub mod card_repository;
pub mod comment_repository;
pub mod event_repository;

pub use board_repository::BoardRepository;
pub use card_repository::{CardFilters, CardRepository, CreateCardInput, UpdateCardInput};
pub use comment_repository::CommentRepository;
pub use event_repository::{CreateEventInput, EventRepository};

pub mod event_actions {
    pub const CARD_CREATED: &str = "card_created";
    pub const CARD_UPDATED: &str = "card_updated";
    pub const CARD_CLOSED: &str = "card_closed";
    pub const CARD_REOPENED: &str = "card_reopened";
    pub const CARD_COLUMN_CHANGED: &str = "card_column_changed";
    pub const CARD_BOARD_CHANGED: &str = "card_board_changed";
    pub const COMMENT_CREATED: &str = "comment_created";
}
