use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::ListBoardCardsInput;
use crate::infrastructure::telegram::bot::BotState;
use crate::infrastructure::telegram::formatters::CardFormatter;

pub async fn handle(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    name: String,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(
            msg.chat.id,
            "Sorry, you are not authorized to use this bot.",
        )
        .await?;
        return Ok(());
    }

    let input = ListBoardCardsInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        board_name: name.clone(),
        limit: Some(20),
    };

    match state.list_board_cards.execute(input).await {
        Ok(output) => {
            let response = if output.cards.is_empty() {
                format!("📁 <b>{}</b>\n\nNo active cards.", output.board_name)
            } else {
                format!(
                    "📁 <b>{}</b> ({} cards)\n\n{}",
                    output.board_name,
                    output.cards.len(),
                    CardFormatter::format_card_list(&output.cards)
                )
            };
            bot.send_message(msg.chat.id, response)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
        }
        Err(e) => {
            tracing::error!("Error listing board cards: {:?}", e);
            bot.send_message(msg.chat.id, format!("Error: {}", e))
                .await?;
        }
    }

    Ok(())
}
