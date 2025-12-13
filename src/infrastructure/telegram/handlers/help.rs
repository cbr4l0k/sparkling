use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::infrastructure::telegram::bot::{BotState, Command};

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

    let help_text = Command::descriptions().to_string();
    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}
