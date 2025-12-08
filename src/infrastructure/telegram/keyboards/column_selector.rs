use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::domain::entities::Column;

/// Create inline keyboard for column selection
pub fn column_selector_keyboard(columns: &[Column], callback_prefix: &str) -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = columns
        .iter()
        .map(|column| {
            vec![InlineKeyboardButton::callback(
                &column.formatted_name(),
                format!("{}:{}", callback_prefix, column.id),
            )]
        })
        .collect();

    InlineKeyboardMarkup::new(buttons)
}
