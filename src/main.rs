use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fizzy_bot::infrastructure::config::AppConfig;
use fizzy_bot::infrastructure::telegram::bot::{create_bot, BotState, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,fizzy_bot=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Fizzy Bot...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = AppConfig::from_env().map_err(|e| anyhow::anyhow!("Config error: {}", e))?;
    tracing::info!("Configuration loaded");

    // Create bot state
    let state = Arc::new(BotState::new(config.clone()));

    // Create bot
    let bot = create_bot(&config);
    tracing::info!("Bot initialized");

    // Build handler
    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command);

    // Start bot
    tracing::info!("Bot is running! Press Ctrl+C to stop.");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<BotState>,
) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    // Authorization check
    if !state.is_authorized(user_id) {
        bot.send_message(
            msg.chat.id,
            "Sorry, you are not authorized to use this bot.",
        )
        .await?;
        return Ok(());
    }

    match cmd {
        Command::Start => {
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
        }

        Command::Help => {
            let help_text = Command::descriptions().to_string();
            bot.send_message(msg.chat.id, help_text).await?;
        }

        Command::Me | Command::MyCards => {
            // TODO: Implement with use case in Phase 2
            bot.send_message(
                msg.chat.id,
                "Your cards will be shown here (coming in Phase 2!)",
            )
            .await?;
        }

        Command::Boards => {
            // TODO: Implement with use case in Phase 2
            bot.send_message(
                msg.chat.id,
                "Board list will be shown here (coming in Phase 2!)",
            )
            .await?;
        }

        Command::Board { name } => {
            // TODO: Implement with use case in Phase 2
            bot.send_message(
                msg.chat.id,
                format!("Cards in board '{}' (coming in Phase 2!)", name),
            )
            .await?;
        }

        Command::Card { number } => {
            // TODO: Implement with use case in Phase 2
            bot.send_message(
                msg.chat.id,
                format!("Card #{} details (coming in Phase 2!)", number),
            )
            .await?;
        }

        Command::Create { title } => {
            if title.trim().is_empty() {
                bot.send_message(msg.chat.id, "Usage: /create <title>")
                    .await?;
            } else {
                // TODO: Implement with use case in Phase 3
                bot.send_message(
                    msg.chat.id,
                    format!("Card '{}' will be created (coming in Phase 3!)", title),
                )
                .await?;
            }
        }

        Command::Close { number } => {
            // TODO: Implement with use case in Phase 3
            bot.send_message(
                msg.chat.id,
                format!("Card #{} will be closed (coming in Phase 3!)", number),
            )
            .await?;
        }

        Command::Reopen { number } => {
            // TODO: Implement with use case in Phase 3
            bot.send_message(
                msg.chat.id,
                format!("Card #{} will be reopened (coming in Phase 3!)", number),
            )
            .await?;
        }

        Command::Comment { number, text } => {
            if text.trim().is_empty() {
                bot.send_message(msg.chat.id, "Usage: /comment <number> <text>")
                    .await?;
            } else {
                // TODO: Implement with use case in Phase 3
                bot.send_message(
                    msg.chat.id,
                    format!("Comment on card #{} (coming in Phase 3!)", number),
                )
                .await?;
            }
        }
    }

    Ok(())
}
