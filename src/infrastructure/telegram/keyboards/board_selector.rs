use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::domain::entities::Board;

/// Create inline keyboard for board selection
pub fn board_selector_keyboard(boards: &[Board], callback_prefix: &str) -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = boards
        .iter()
        .map(|board| {
            vec![InlineKeyboardButton::callback(
                &board.name,
                format!("{}:{}", callback_prefix, board.id),
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(buttons)
}
