use std::sync::Arc;
use teloxide::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sparkling::infrastructure::config::AppConfig;
use sparkling::infrastructure::persistence::{
    create_pool, SqliteBoardRepository, SqliteCardRepository,
};
use sparkling::infrastructure::telegram::bot::{create_bot, BotState, Command};
use sparkling::infrastructure::telegram::handlers;

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
        Command::Start => handlers::start::handle(bot, msg, state).await?,
        Command::Help => handlers::help::handle(bot, msg, state).await?,
        Command::Me | Command::MyCards => handlers::my_cards::handle(bot, msg, state).await?,
        Command::Boards => handlers::boards::handle(bot, msg, state).await?,
        Command::Board { name } => handlers::board::handle(bot, msg, state, name).await?,
        Command::Card { number } => handlers::card::handle(bot, msg, state, number).await?,
        Command::Create { title } => handlers::create::handle(bot, msg, state, title).await?,
        Command::Close { number } => handlers::close::handle(bot, msg, state, number).await?,
        Command::Reopen { number } => handlers::reopen::handle(bot, msg, state, number).await?,
        Command::Comment { number, text } => {
            handlers::comment::handle(bot, msg, state, number, text).await?
        }
    }

    Ok(())
}
