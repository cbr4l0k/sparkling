use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::GetCardDetailsInput;
use crate::infrastructure::telegram::bot::BotState;
use crate::infrastructure::telegram::formatters::CardFormatter;

pub async fn handle(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
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

    let input = GetCardDetailsInput {
        account_id: state.account_id(),
        card_number: number,
    };

    match state.get_card_details.execute(input).await {
        Ok(card) => {
            let response = CardFormatter::format_card(&card, state.base_url());
            bot.send_message(msg.chat.id, response)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
        }
        Err(e) => {
            tracing::error!("Error getting card details: {:?}", e);
            bot.send_message(msg.chat.id, format!("Error: {}", e))
                .await?;
        }
    }

    Ok(())
}
