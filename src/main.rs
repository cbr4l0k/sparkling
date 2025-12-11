use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sparkling::application::use_cases::{
    GetCardDetailsInput, ListBoardCardsInput, ListBoardsInput, ListMyCardsInput,
};
use sparkling::infrastructure::config::AppConfig;
use sparkling::infrastructure::persistence::{
    create_pool, SqliteBoardRepository, SqliteCardRepository,
};
use sparkling::infrastructure::telegram::bot::{create_bot, BotState, Command};
use sparkling::infrastructure::telegram::formatters::{BoardFormatter, CardFormatter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sparkling=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Fizzy Bot...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = AppConfig::from_env().map_err(|e| anyhow::anyhow!("Config error: {}", e))?;
    tracing::info!("Configuration loaded");

    // Create database pool
    let pool = create_pool(&config).await?;
    tracing::info!("Database pool created");

    // Create repositories
    let card_repository = Arc::new(SqliteCardRepository::new(pool.clone()));
    let board_repository = Arc::new(SqliteBoardRepository::new(pool.clone()));

    // Create bot state with use cases
    let state = Arc::new(BotState::new(
        config.clone(),
        card_repository,
        board_repository,
    ));

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
            let input = ListMyCardsInput {
                account_id: state.account_id(),
                user_id: state.user_id(),
                include_closed: false,
                limit: Some(20),
            };

            match state.list_my_cards.execute(input).await {
                Ok(output) => {
                    let response = if output.cards.is_empty() {
                        "📋 You have no assigned cards.".to_string()
                    } else {
                        format!(
                            "📋 <b>Your Cards</b> ({})\n\n{}",
                            output.cards.len(),
                            CardFormatter::format_card_list(&output.cards)
                        )
                    };
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Error listing cards: {:?}", e);
                    bot.send_message(msg.chat.id, format!("Error: {}", e))
                        .await?;
                }
            }
        }

        Command::Boards => {
            let input = ListBoardsInput {
                account_id: state.account_id(),
                user_id: state.user_id(),
            };

            match state.list_boards.execute(input).await {
                Ok(output) => {
                    let response = if output.boards.is_empty() {
                        "📁 No boards found.".to_string()
                    } else {
                        format!(
                            "📁 <b>Your Boards</b> ({})\n\n{}",
                            output.boards.len(),
                            BoardFormatter::format_board_list(&output.boards)
                        )
                    };
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Error listing boards: {:?}", e);
                    bot.send_message(msg.chat.id, format!("Error: {}", e))
                        .await?;
                }
            }
        }

        Command::Board { name } => {
            let input = ListBoardCardsInput {
                account_id: state.account_id(),
                user_id: state.user_id(),
                board_name: name.clone(),
                limit: Some(20),
            };

            match state.list_board_cards.execute(input).await {
                Ok(output) => {
                    let response = if output.cards.is_empty() {
                        format!("📁 <b>{}</b>\n\nNo active cards.", output.board_name)
                    } else {
                        format!(
                            "📁 <b>{}</b> ({} cards)\n\n{}",
                            output.board_name,
                            output.cards.len(),
                            CardFormatter::format_card_list(&output.cards)
                        )
                    };
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Error listing board cards: {:?}", e);
                    bot.send_message(msg.chat.id, format!("Error: {}", e))
                        .await?;
                }
            }
        }

        Command::Card { number } => {
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
