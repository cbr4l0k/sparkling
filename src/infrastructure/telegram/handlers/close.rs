use std::sync::Arc;
use teloxide::prelude::*;

use crate::infrastructure::telegram::bot::BotState;

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

    // TODO: Implement with use case in Phase 3
    bot.send_message(
        msg.chat.id,
        format!("Card #{} will be closed (coming in Phase 3!)", number),
    )
    .await?;

    Ok(())
}
