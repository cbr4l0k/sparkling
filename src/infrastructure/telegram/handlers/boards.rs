use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::ListBoardsInput;
use crate::infrastructure::telegram::bot::BotState;
use crate::infrastructure::telegram::formatters::BoardFormatter;

pub async fn handle(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(
            msg.chat.id,
            "Sorry, you are not authorized to use this bot.",
        )
        .await?;
        return Ok(());
    }

    let input = ListBoardsInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
    };

    match state.list_boards.execute(input).await {
        Ok(output) => {
            let response = if output.boards.is_empty() {
                "📁 No boards found.".to_string()
            } else {
                format!(
                    "📁 <b>Your Boards</b> ({})\n\n{}",
                    output.boards.len(),
                    BoardFormatter::format_board_list(&output.boards)
                )
            };
            bot.send_message(msg.chat.id, response)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
        }
        Err(e) => {
            tracing::error!("Error listing boards: {:?}", e);
            bot.send_message(msg.chat.id, format!("Error: {}", e))
                .await?;
        }
    }

    Ok(())
}
