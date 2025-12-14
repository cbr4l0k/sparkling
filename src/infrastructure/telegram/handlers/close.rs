use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::CloseCardInput;
use crate::infrastructure::telegram::bot::BotState;

pub async fn handle(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
) -> ResponseResult<()> {
    let input = CloseCardInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        card_number: number,
    };

    match state.close_card.execute(input).await {
        Ok(()) => {
            bot.send_message(msg.chat.id, format!("Card #{} has been closed.", number))
                .await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Failed to close card: {}", e))
                .await?;
        }
    }

    Ok(())
}
