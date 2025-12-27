# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sparkling is a Telegram bot that interfaces with [Fizzy](https://www.fizzy.do/), a self-hosted Kanban app. It allows managing Fizzy cards, boards, and comments directly from Telegram.

## Build & Run Commands

```bash
# Build
cargo build
cargo build --release

# Run (requires .env file with configuration)
cargo run

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Environment Variables

Required in `.env`:
- `TELEGRAM_BOT_TOKEN` - Telegram bot API token
- `TELEGRAM_ALLOWED_USER_IDS` - Comma-separated list of allowed Telegram user IDs
- `DATABASE_PATH` - Path to Fizzy's SQLite database (e.g., `storage/development.sqlite3`)
- `FIZZY_ACCOUNT_ID` - UUID of the Fizzy account
- `FIZZY_USER_ID` - UUID of the Fizzy user
- `FIZZY_DEFAULT_BOARD_ID` - UUID of the default board for card creation

Optional:
- `DATABASE_MAX_CONNECTIONS` - SQLite connection pool size (default: 5)
- `FIZZY_BASE_URL` - Base URL for Fizzy web UI links

## Architecture

The project follows **Hexagonal Architecture (Ports & Adapters)** with clear layer separation:

```
src/
├── domain/           # Core business logic (no dependencies)
│   ├── entities/     # Card, Board, Column, Comment, User
│   ├── value_objects/# FizzyId, CardStatus
│   ├── ports/        # Repository traits (interfaces)
│   └── errors.rs
├── application/      # Use cases (orchestration layer)
│   └── use_cases/    # CreateCard, CloseCard, MoveCard, AddComment, etc.
├── infrastructure/   # External adapters
│   ├── persistence/  # SQLite repositories (named mysql_* for legacy reasons)
│   ├── telegram/     # Bot handlers, formatters, keyboards
│   └── config/       # Environment configuration
└── shared/           # Cross-cutting concerns
```

### Key Patterns

- **Repository Pattern**: `domain/ports/` defines traits, `infrastructure/persistence/` implements them
- **Use Case Pattern**: Each user action is a separate use case in `application/use_cases/`
- **BotState**: Centralized dependency injection container (`infrastructure/telegram/bot.rs`)
- **Callbacks**: Telegram inline keyboard interactions handled in `handlers/callbacks.rs`

### Database

Sparkling directly reads/writes Fizzy's SQLite database. Key tables: `cards`, `boards`, `columns`, `comments`, `events`. Cards use `number` (human-readable integer) for user interaction, but UUIDs (`FizzyId`) internally.

### Telegram Commands

Defined in `Command` enum (`infrastructure/telegram/bot.rs`):
- `/me`, `/mycards` - List assigned cards
- `/boards` - List boards
- `/board <name>` - Show board cards
- `/card <number>` - Show card details
- `/create <title>` - Create card
- `/close <number>` / `/reopen <number>` - Close/reopen card
- `/comment <number> <text>` - Add comment

## Database File Permissions

When modifying the database directly:
```bash
chmod 755 storage/
sudo chmod 644 development.sqlite3
sudo chown $USER:$USER development*
```
