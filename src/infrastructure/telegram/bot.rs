use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::{
    GetCardDetailsUseCase, ListBoardCardsUseCase, ListBoardsUseCase, ListMyCardsUseCase,
};
use crate::domain::ports::{BoardRepository, CardRepository};
use crate::domain::value_objects::FizzyId;
use crate::infrastructure::config::AppConfig;

/// Shared state for the bot handlers
#[derive(Clone)]
pub struct BotState {
    pub config: Arc<AppConfig>,
    // Use cases for Phase 2
    pub list_my_cards: Arc<ListMyCardsUseCase>,
    pub get_card_details: Arc<GetCardDetailsUseCase>,
    pub list_boards: Arc<ListBoardsUseCase>,
    pub list_board_cards: Arc<ListBoardCardsUseCase>,
}

impl BotState {
    pub fn new(
        config: AppConfig,
        card_repository: Arc<dyn CardRepository>,
        board_repository: Arc<dyn BoardRepository>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            list_my_cards: Arc::new(ListMyCardsUseCase::new(card_repository.clone())),
            get_card_details: Arc::new(GetCardDetailsUseCase::new(card_repository.clone())),
            list_boards: Arc::new(ListBoardsUseCase::new(board_repository.clone())),
            list_board_cards: Arc::new(ListBoardCardsUseCase::new(
                card_repository,
                board_repository,
            )),
        }
    }

    /// Check if a Telegram user is authorized to use the bot
    pub fn is_authorized(&self, user_id: i64) -> bool {
        self.config.telegram.is_user_allowed(user_id)
    }

    /// Get the configured account ID
    pub fn account_id(&self) -> FizzyId {
        FizzyId::new(self.config.fizzy.account_id.clone())
    }

    /// Get the configured user ID (for single-user mode)
    pub fn user_id(&self) -> FizzyId {
        FizzyId::new(self.config.fizzy.user_id.clone())
    }

    /// Get the base URL for Fizzy web UI
    pub fn base_url(&self) -> Option<&str> {
        self.config.fizzy.base_url.as_deref()
    }
}

/// Create and configure the Telegram bot
pub fn create_bot(config: &AppConfig) -> Bot {
    Bot::new(&config.telegram.bot_token)
}

#[derive(Clone, teloxide::macros::BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Start the bot and show welcome message")]
    Start,

    #[command(description = "Show this help message")]
    Help,

    #[command(description = "List your assigned cards")]
    Me,

    #[command(description = "List your assigned cards (alias)")]
    MyCards,

    #[command(description = "List accessible boards")]
    Boards,

    #[command(description = "Show cards in a board")]
    Board { name: String },

    #[command(description = "Show card details")]
    Card { number: i64 },

    #[command(description = "Create a new card")]
    Create { title: String },

    #[command(description = "Close a card")]
    Close { number: i64 },

    #[command(description = "Reopen a closed card")]
    Reopen { number: i64 },

    #[command(description = "Add a comment to a card", parse_with = "split")]
    Comment { number: i64, text: String },
}
