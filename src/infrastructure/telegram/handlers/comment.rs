use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::AddCommentInput;
use crate::infrastructure::telegram::bot::BotState;

pub async fn handle(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
    text: String,
) -> ResponseResult<()> {
    if text.trim().is_empty() {
        bot.send_message(msg.chat.id, "Usage: /comment <number> <text>")
            .await?;
        return Ok(());
    }

    let input = AddCommentInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        card_number: number,
        content: text.trim().to_string(),
    };

    match state.add_comment.execute(input).await {
        Ok(_comment) => {
            bot.send_message(
                msg.chat.id,
                format!("Comment added to card #{}.", number),
            )
            .await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Failed to add comment: {}", e))
                .await?;
        }
    }

    Ok(())
}
