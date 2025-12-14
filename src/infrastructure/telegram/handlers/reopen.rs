use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::ReopenCardInput;
use crate::infrastructure::telegram::bot::BotState;

pub async fn handle(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
) -> ResponseResult<()> {
    let input = ReopenCardInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        card_number: number,
    };

    match state.reopen_card.execute(input).await {
        Ok(()) => {
            bot.send_message(msg.chat.id, format!("Card #{} has been reopened.", number))
                .await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Failed to reopen card: {}", e))
                .await?;
        }
    }

    Ok(())
}
