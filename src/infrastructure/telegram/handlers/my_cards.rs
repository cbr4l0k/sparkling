use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::ListMyCardsInput;
use crate::infrastructure::telegram::bot::BotState;
use crate::infrastructure::telegram::formatters::CardFormatter;

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

    let input = ListMyCardsInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        include_closed: false,
        limit: Some(20),
    };

    match state.list_my_cards.execute(input).await {
        Ok(output) => {
            let response = if output.cards.is_empty() {
                "📋 You have no assigned cards.".to_string()
            } else {
                format!(
                    "📋 <b>Your Cards</b> ({})\n\n{}",
                    output.cards.len(),
                    CardFormatter::format_card_list(&output.cards)
                )
            };
            bot.send_message(msg.chat.id, response)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
        }
        Err(e) => {
            tracing::error!("Error listing cards: {:?}", e);
            bot.send_message(msg.chat.id, format!("Error: {}", e))
                .await?;
        }
    }

    Ok(())
}
