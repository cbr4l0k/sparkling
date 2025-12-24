use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::{CloseCardInput, GetCardDetailsInput, MoveCardInput};
use crate::domain::value_objects::FizzyId;
use crate::infrastructure::telegram::bot::BotState;
use crate::infrastructure::telegram::keyboards::column_selector_keyboard;

/// Handle callback queries from inline keyboard buttons
pub async fn handle_callback(
    bot: Bot,
    query: CallbackQuery,
    state: Arc<BotState>,
) -> ResponseResult<()> {
    let user_id = query.from.id.0 as i64;

    // Authorization check
    if !state.is_authorized(user_id) {
        bot.answer_callback_query(query.id.clone())
            .text("You are not authorized to use this bot.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let Some(data) = &query.data else {
        bot.answer_callback_query(query.id.clone()).await?;
        return Ok(());
    };

    // Parse callback data (format: "action:param" or "action:param1:param2")
    let parts: Vec<&str> = data.split(':').collect();

    match parts.as_slice() {
        ["close", card_number] => {
            handle_close(&bot, &query, &state, card_number).await?;
        }
        ["comment", card_number] => {
            handle_comment_prompt(&bot, &query, card_number).await?;
        }
        ["move", card_number] => {
            handle_move_select_column(&bot, &query, &state, card_number).await?;
        }
        ["move_to", card_number, column_id] => {
            handle_move_to_column(&bot, &query, &state, card_number, column_id).await?;
        }
        _ => {
            bot.answer_callback_query(query.id.clone())
                .text("Unknown action")
                .await?;
        }
    }

    Ok(())
}

/// Handle close card callback
async fn handle_close(
    bot: &Bot,
    query: &CallbackQuery,
    state: &Arc<BotState>,
    card_number_str: &str,
) -> ResponseResult<()> {
    let card_number: i64 = match card_number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            bot.answer_callback_query(query.id.clone())
                .text("Invalid card number")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    let input = CloseCardInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        card_number,
    };

    match state.close_card.execute(input).await {
        Ok(()) => {
            bot.answer_callback_query(query.id.clone())
                .text(format!("Card #{} closed", card_number))
                .await?;

            // Send confirmation message
            if let Some(msg) = &query.message {
                bot.send_message(msg.chat().id, format!("✅ Card #{} has been closed.", card_number))
                    .await?;
            }
        }
        Err(e) => {
            bot.answer_callback_query(query.id.clone())
                .text(format!("Failed to close: {}", e))
                .show_alert(true)
                .await?;
        }
    }

    Ok(())
}

/// Handle comment prompt - direct user to use /comment command
async fn handle_comment_prompt(
    bot: &Bot,
    query: &CallbackQuery,
    card_number_str: &str,
) -> ResponseResult<()> {
    bot.answer_callback_query(query.id.clone()).await?;

    if let Some(msg) = &query.message {
        bot.send_message(
            msg.chat().id,
            format!(
                "💬 To add a comment to card #{}, use:\n<code>/comment {} your comment text</code>",
                card_number_str, card_number_str
            ),
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    }

    Ok(())
}

/// Handle move card - show column selector
async fn handle_move_select_column(
    bot: &Bot,
    query: &CallbackQuery,
    state: &Arc<BotState>,
    card_number_str: &str,
) -> ResponseResult<()> {
    let card_number: i64 = match card_number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            bot.answer_callback_query(query.id.clone())
                .text("Invalid card number")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    // Get card details to find board
    let input = GetCardDetailsInput {
        account_id: state.account_id(),
        card_number,
    };

    let card = match state.get_card_details.execute(input).await {
        Ok(card) => card,
        Err(e) => {
            bot.answer_callback_query(query.id.clone())
                .text(format!("Card not found: {}", e))
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    // Get columns for the board
    let columns = match state
        .board_repository
        .get_columns(&state.account_id(), &card.board_id)
        .await
    {
        Ok(cols) => cols,
        Err(e) => {
            bot.answer_callback_query(query.id.clone())
                .text(format!("Failed to get columns: {}", e))
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    if columns.is_empty() {
        bot.answer_callback_query(query.id.clone())
            .text("No columns available")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    bot.answer_callback_query(query.id.clone()).await?;

    // Build callback prefix: move_to:<card_number>
    let callback_prefix = format!("move_to:{}", card_number);
    let keyboard = column_selector_keyboard(&columns, &callback_prefix);

    if let Some(msg) = &query.message {
        bot.send_message(
            msg.chat().id,
            format!("📁 Select column to move card #{} to:", card_number),
        )
        .reply_markup(keyboard)
        .await?;
    }

    Ok(())
}

/// Handle move to specific column
async fn handle_move_to_column(
    bot: &Bot,
    query: &CallbackQuery,
    state: &Arc<BotState>,
    card_number_str: &str,
    column_id_str: &str,
) -> ResponseResult<()> {
    let card_number: i64 = match card_number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            bot.answer_callback_query(query.id.clone())
                .text("Invalid card number")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    let input = MoveCardInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        card_number,
        column_id: FizzyId::new(column_id_str.to_string()),
    };

    match state.move_card.execute(input).await {
        Ok(card) => {
            let column_name = card.column_name.as_deref().unwrap_or("new column");
            bot.answer_callback_query(query.id.clone())
                .text(format!("Card #{} moved", card_number))
                .await?;

            if let Some(msg) = &query.message {
                bot.send_message(
                    msg.chat().id,
                    format!("📁 Card #{} moved to {}.", card_number, column_name),
                )
                .await?;
            }
        }
        Err(e) => {
            bot.answer_callback_query(query.id.clone())
                .text(format!("Failed to move: {}", e))
                .show_alert(true)
                .await?;
        }
    }

    Ok(())
}
