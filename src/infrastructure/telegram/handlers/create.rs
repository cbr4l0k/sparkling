use std::sync::Arc;
use teloxide::prelude::*;

use crate::application::use_cases::CreateCardUseCaseInput;
use crate::infrastructure::telegram::bot::BotState;

pub async fn handle(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    title: String,
) -> ResponseResult<()> {
    if title.trim().is_empty() {
        bot.send_message(msg.chat.id, "Usage: /create <title>")
            .await?;
        return Ok(());
    }

    let input = CreateCardUseCaseInput {
        account_id: state.account_id(),
        user_id: state.user_id(),
        board_id: state.default_board_id(),
        title: title.trim().to_string(),
        description: None,
    };

    match state.create_card.execute(input).await {
        Ok(card) => {
            let mut response = format!("Created card #{}: {}", card.number, card.title);
            if let Some(base_url) = state.base_url() {
                response.push_str(&format!("\n{}/cards/{}", base_url, card.number));
            }
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("Failed to create card: {}", e))
                .await?;
        }
    }

    Ok(())
}
