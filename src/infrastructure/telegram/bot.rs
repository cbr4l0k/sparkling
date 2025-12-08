use crate::infrastructure::config::AppConfig;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

/// Shared state for the bot handlers
#[derive(Clone)]
pub struct BotState {
    pub config: Arc<AppConfig>,
}

impl BotState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Check if a Telegram user is authorized to use the bot
    pub fn is_authorized(&self, user_id: i64) -> bool {
        self.config.telegram.is_user_allowed(user_id)
    }
}

/// Create and configure the Telegram bot
pub fn create_bot(config: &AppConfig) -> Bot {
    Bot::new(&config.telegram.bot_token)
}

#[derive(Clone, teloxide::macros::BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Start the bot and show welcome message")]
    Start,

    #[command(description = "Show this help message")]
    Help,

    #[command(description = "List your assigned cards")]
    Me,

    #[command(description = "List your assigned cards (alias)")]
    MyCards,

    #[command(description = "List accessible boards")]
    Boards,

    #[command(description = "Show cards in a board")]
    Board { name: String },

    #[command(description = "Show card details")]
    Card { number: i64 },

    #[command(description = "Create a new card")]
    Create { title: String },

    #[command(description = "Close a card")]
    Close { number: i64 },

    #[command(description = "Reopen a closed card")]
    Reopen { number: i64 },

    #[command(description = "Add a comment to a card", parse_with = "split")]
    Comment { number: i64, text: String },
}

/// Handler for /start command
async fn handle_start(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
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

/// Handler for /help command
async fn handle_help(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
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

/// Handler for /me and /my_cards commands
async fn handle_me(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 2
    bot.send_message(msg.chat.id, "Your cards will be shown here (coming soon!)")
        .await?;
    Ok(())
}

/// Handler for /boards command
async fn handle_boards(bot: Bot, msg: Message, state: Arc<BotState>) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 2
    bot.send_message(msg.chat.id, "Board list will be shown here (coming soon!)")
        .await?;
    Ok(())
}

/// Handler for /board command
async fn handle_board(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    name: String,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 2
    bot.send_message(
        msg.chat.id,
        format!(
            "Cards in board '{}' will be shown here (coming soon!)",
            name
        ),
    )
    .await?;
    Ok(())
}

/// Handler for /card command
async fn handle_card(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 2
    bot.send_message(
        msg.chat.id,
        format!("Card #{} details will be shown here (coming soon!)", number),
    )
    .await?;
    Ok(())
}

/// Handler for /create command
async fn handle_create(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    title: String,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    if title.trim().is_empty() {
        bot.send_message(msg.chat.id, "Usage: /create <title>")
            .await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 3
    bot.send_message(
        msg.chat.id,
        format!("Card '{}' will be created here (coming soon!)", title),
    )
    .await?;
    Ok(())
}

/// Handler for /close command
async fn handle_close(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 3
    bot.send_message(
        msg.chat.id,
        format!("Card #{} will be closed here (coming soon!)", number),
    )
    .await?;
    Ok(())
}

/// Handler for /reopen command
async fn handle_reopen(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    number: i64,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 3
    bot.send_message(
        msg.chat.id,
        format!("Card #{} will be reopened here (coming soon!)", number),
    )
    .await?;
    Ok(())
}

/// Handler for /comment command
async fn handle_comment(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    (number, text): (i64, String),
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !state.is_authorized(user_id) {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }

    if text.trim().is_empty() {
        bot.send_message(msg.chat.id, "Usage: /comment <number> <text>")
            .await?;
        return Ok(());
    }

    // TODO: Implement with use case in Phase 3
    bot.send_message(
        msg.chat.id,
        format!(
            "Comment on card #{} will be added here (coming soon!)",
            number
        ),
    )
    .await?;
    Ok(())
}
