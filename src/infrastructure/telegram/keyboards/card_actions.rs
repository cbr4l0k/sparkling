use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Create inline keyboard with card action buttons
pub fn card_actions_keyboard(card_number: i64) -> InlineKeyboardMarkup {
    let buttons = vec![
        vec![
            InlineKeyboardButton::callback("✅ Close", format!("close:{}", card_number)),
            InlineKeyboardButton::callback("💬 Comment", format!("comment:{}", card_number)),
        ],
        vec![InlineKeyboardButton::callback(
            "📁 Move",
            format!("move:{}", card_number),
        )],
    ];

    InlineKeyboardMarkup::new(buttons)
}
