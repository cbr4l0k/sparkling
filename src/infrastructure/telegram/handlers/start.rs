use std::sync::Arc;
use teloxide::prelude::*;

use crate::infrastructure::telegram::bot::BotState;

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

    let welcome = r#"Welcome to Fizzy Bot!

I help you manage your Fizzy cards from Telegram.

Quick commands:
/me - Your assigned cards
/boards - List boards
/card 123 - View card #123
/create My new task - Create a card
/close 123 - Close card #123

Type /help for all commands."#;

    bot.send_message(msg.chat.id, welcome).await?;
    Ok(())
}
