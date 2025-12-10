use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub telegram: TelegramConfig,
    pub database: DatabaseConfig,
    pub fizzy: FizzyConfig,
}

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_user_ids: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct FizzyConfig {
    pub account_id: String,
    pub user_id: String,
    pub default_board_id: String,
    pub base_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            telegram: TelegramConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            fizzy: FizzyConfig::from_env()?,
        })
    }
}

impl TelegramConfig {
    fn from_env() -> Result<Self, String> {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| "TELEGRAM_BOT_TOKEN not set")?;

        let allowed_user_ids = env::var("TELEGRAM_ALLOWED_USER_IDS")
            .map_err(|_| "TELEGRAM_ALLOWED_USER_IDS not set")?
            .split(',')
            .map(|s| s.trim().parse::<i64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Invalid TELEGRAM_ALLOWED_USER_IDS format")?;

        Ok(Self {
            bot_token,
            allowed_user_ids,
        })
    }

    pub fn is_user_allowed(&self, user_id: i64) -> bool {
        self.allowed_user_ids.contains(&user_id)
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self, String> {
        Ok(Self {
            path: env::var("DATABASE_PATH")
                .map_err(|_| "DATABASE_PATH not set")?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| "Invalid DATABASE_MAX_CONNECTIONS")?,
        })
    }

    pub fn connection_string(&self) -> String {
        format!("sqlite://{}?mode=ro", self.path)
    }
}

impl FizzyConfig {
    fn from_env() -> Result<Self, String> {
        Ok(Self {
            account_id: env::var("FIZZY_ACCOUNT_ID")
                .map_err(|_| "FIZZY_ACCOUNT_ID not set")?,
            user_id: env::var("FIZZY_USER_ID")
                .map_err(|_| "FIZZY_USER_ID not set")?,
            default_board_id: env::var("FIZZY_DEFAULT_BOARD_ID")
                .map_err(|_| "FIZZY_DEFAULT_BOARD_ID not set")?,
            base_url: env::var("FIZZY_BASE_URL").ok(),
        })
    }
}
