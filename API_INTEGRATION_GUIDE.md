# Fizzy API Integration Guide for Telegram Bot

This guide explains how to integrate a Telegram bot with your Fizzy instance using **TypeScript**.

## Current State: No Dedicated API

Fizzy is built as a modern Rails application using **Turbo/Hotwire**, not a traditional REST API. However, there are several ways to integrate with it.

---

## Option 1: Direct Database Access (Recommended for Self-Hosted)

Since you're self-hosting Fizzy, the **simplest and most reliable** approach is to access the database directly from your Telegram bot.

### Advantages
- Full access to all data
- No authentication complexity
- Fast and efficient
- No need to modify Fizzy codebase
- Use the DATABASE_SCHEMA.md for reference

### Implementation

Your bot can connect to the same MySQL database and query it directly:

```typescript
import mysql from 'mysql2/promise';
import { Telegraf } from 'telegraf';

interface Card {
  number: number;
  title: string;
  status: string;
  due_on: Date | null;
  board_name: string;
}

class FizzyDB {
  private pool: mysql.Pool;

  constructor(config: mysql.PoolOptions) {
    this.pool = mysql.createPool(config);
  }

  async getUserCards(accountId: string, userId: string): Promise<Card[]> {
    const [rows] = await this.pool.query<mysql.RowDataPacket[]>(
      `
      SELECT c.number, c.title, c.status, c.due_on, b.name as board_name
      FROM cards c
      JOIN boards b ON c.board_id = b.id
      JOIN assignments a ON c.id = a.card_id
      WHERE c.account_id = ?
        AND a.assignee_id = ?
        AND c.status NOT IN ('closed', 'not_now')
      ORDER BY c.last_active_at DESC
      LIMIT 10
      `,
      [accountId, userId]
    );

    return rows as Card[];
  }

  async searchCards(accountId: string, query: string): Promise<Card[]> {
    // Use the sharded search tables
    // Shard determined by: CRC32(account_id) % 16
    const shard = this.calculateShard(accountId);

    const [rows] = await this.pool.query<mysql.RowDataPacket[]>(
      `
      SELECT c.number, c.title, c.status, c.due_on, b.name as board_name
      FROM search_records_${shard} sr
      JOIN cards c ON sr.card_id = c.id
      JOIN boards b ON c.board_id = b.id
      WHERE sr.account_id = ?
        AND MATCH(sr.title, sr.content) AGAINST(? IN NATURAL LANGUAGE MODE)
      LIMIT 20
      `,
      [accountId, query]
    );

    return rows as Card[];
  }

  async createCard(
    accountId: string,
    boardId: string,
    userId: string,
    title: string
  ): Promise<number> {
    const connection = await this.pool.getConnection();

    try {
      await connection.beginTransaction();

      // Get next card number
      const [result] = await connection.query<mysql.RowDataPacket[]>(
        `
        UPDATE accounts
        SET cards_count = cards_count + 1
        WHERE id = ?
        `,
        [accountId]
      );

      const [accountRows] = await connection.query<mysql.RowDataPacket[]>(
        'SELECT cards_count FROM accounts WHERE id = ?',
        [accountId]
      );

      const cardNumber = accountRows[0].cards_count;

      // Create card
      await connection.query(
        `
        INSERT INTO cards (
          id, account_id, board_id, creator_id, number,
          title, status, last_active_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, 'drafted', NOW(), NOW(), NOW())
        `,
        [this.generateUUID(), accountId, boardId, userId, cardNumber, title]
      );

      await connection.commit();
      return cardNumber;
    } catch (error) {
      await connection.rollback();
      throw error;
    } finally {
      connection.release();
    }
  }

  async getCardDetails(accountId: string, cardNumber: number) {
    const [rows] = await this.pool.query<mysql.RowDataPacket[]>(
      `
      SELECT
        c.*,
        b.name as board_name,
        col.name as column_name,
        col.color as column_color,
        u.name as creator_name,
        GROUP_CONCAT(DISTINCT t.title) as tags,
        GROUP_CONCAT(DISTINCT assignee.name) as assignees
      FROM cards c
      JOIN boards b ON c.board_id = b.id
      LEFT JOIN columns col ON c.column_id = col.id
      JOIN users u ON c.creator_id = u.id
      LEFT JOIN taggings tg ON c.id = tg.card_id
      LEFT JOIN tags t ON tg.tag_id = t.id
      LEFT JOIN assignments a ON c.id = a.card_id
      LEFT JOIN users assignee ON a.assignee_id = assignee.id
      WHERE c.account_id = ? AND c.number = ?
      GROUP BY c.id
      `,
      [accountId, cardNumber]
    );

    return rows[0] || null;
  }

  async addComment(
    accountId: string,
    cardId: string,
    userId: string,
    content: string
  ): Promise<string> {
    const commentId = this.generateUUID();

    await this.pool.query(
      `
      INSERT INTO comments (id, account_id, card_id, creator_id, created_at, updated_at)
      VALUES (?, ?, ?, ?, NOW(), NOW())
      `,
      [commentId, accountId, cardId, userId]
    );

    // Add rich text content
    await this.pool.query(
      `
      INSERT INTO action_text_rich_texts (
        id, account_id, record_type, record_id, name, body, created_at, updated_at
      ) VALUES (?, ?, 'Comment', ?, 'content', ?, NOW(), NOW())
      `,
      [this.generateUUID(), accountId, commentId, content]
    );

    return commentId;
  }

  async closeCard(accountId: string, cardId: string, userId: string): Promise<void> {
    const connection = await this.pool.getConnection();

    try {
      await connection.beginTransaction();

      // Update card status
      await connection.query(
        'UPDATE cards SET status = ? WHERE id = ? AND account_id = ?',
        ['closed', cardId, accountId]
      );

      // Create closure record
      await connection.query(
        `
        INSERT INTO closures (id, account_id, card_id, user_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, NOW(), NOW())
        `,
        [this.generateUUID(), accountId, cardId, userId]
      );

      await connection.commit();
    } catch (error) {
      await connection.rollback();
      throw error;
    } finally {
      connection.release();
    }
  }

  private calculateShard(accountId: string): number {
    // CRC32 implementation for sharding
    const crc32 = require('crc-32');
    const hash = crc32.str(accountId);
    return Math.abs(hash) % 16;
  }

  private generateUUID(): string {
    // Use uuid library for UUIDv7 generation
    const { v7: uuidv7 } = require('uuid');
    return uuidv7();
  }
}

// Initialize bot
const bot = new Telegraf(process.env.BOT_TOKEN!);

const db = new FizzyDB({
  host: 'localhost',
  user: 'fizzy_user',
  password: 'your_password',
  database: 'fizzy_production',
  waitForConnections: true,
  connectionLimit: 10,
});

// Bot commands
bot.command('my_cards', async (ctx) => {
  const accountId = process.env.ACCOUNT_ID!;
  const userId = process.env.USER_ID!;

  try {
    const cards = await db.getUserCards(accountId, userId);

    if (cards.length === 0) {
      return ctx.reply('You have no active cards.');
    }

    const message = cards
      .map(
        (card) =>
          `#${card.number} - ${card.title}\n` +
          `   📋 ${card.board_name} | Status: ${card.status}` +
          (card.due_on ? `\n   📅 Due: ${card.due_on.toISOString().split('T')[0]}` : '')
      )
      .join('\n\n');

    ctx.reply(`📋 Your Cards:\n\n${message}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error fetching cards.');
  }
});

bot.command('search', async (ctx) => {
  const query = ctx.message.text.split(' ').slice(1).join(' ');

  if (!query) {
    return ctx.reply('Usage: /search <query>');
  }

  try {
    const cards = await db.searchCards(process.env.ACCOUNT_ID!, query);

    if (cards.length === 0) {
      return ctx.reply('No cards found.');
    }

    const message = cards
      .map((card) => `#${card.number} - ${card.title} (${card.board_name})`)
      .join('\n');

    ctx.reply(`🔍 Search Results:\n\n${message}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error searching cards.');
  }
});

bot.command('create', async (ctx) => {
  const title = ctx.message.text.split(' ').slice(1).join(' ');

  if (!title) {
    return ctx.reply('Usage: /create <card title>');
  }

  try {
    const cardNumber = await db.createCard(
      process.env.ACCOUNT_ID!,
      process.env.BOARD_ID!,
      process.env.USER_ID!,
      title
    );

    ctx.reply(`✅ Created card #${cardNumber}: ${title}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error creating card.');
  }
});

bot.command('card', async (ctx) => {
  const cardNumber = parseInt(ctx.message.text.split(' ')[1]);

  if (!cardNumber) {
    return ctx.reply('Usage: /card <number>');
  }

  try {
    const card = await db.getCardDetails(process.env.ACCOUNT_ID!, cardNumber);

    if (!card) {
      return ctx.reply('Card not found.');
    }

    const message = [
      `📋 Card #${card.number}`,
      `\n**${card.title}**`,
      `\nBoard: ${card.board_name}`,
      card.column_name ? `Column: ${card.column_name}` : '',
      `Status: ${card.status}`,
      `Creator: ${card.creator_name}`,
      card.assignees ? `Assignees: ${card.assignees}` : '',
      card.tags ? `Tags: ${card.tags}` : '',
      card.due_on ? `\nDue: ${new Date(card.due_on).toISOString().split('T')[0]}` : '',
    ]
      .filter(Boolean)
      .join('\n');

    ctx.reply(message);
  } catch (error) {
    console.error(error);
    ctx.reply('Error fetching card details.');
  }
});

bot.command('close', async (ctx) => {
  const cardNumber = parseInt(ctx.message.text.split(' ')[1]);

  if (!cardNumber) {
    return ctx.reply('Usage: /close <card_number>');
  }

  try {
    const card = await db.getCardDetails(process.env.ACCOUNT_ID!, cardNumber);

    if (!card) {
      return ctx.reply('Card not found.');
    }

    await db.closeCard(process.env.ACCOUNT_ID!, card.id, process.env.USER_ID!);

    ctx.reply(`✅ Closed card #${cardNumber}: ${card.title}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error closing card.');
  }
});

bot.launch();

// Enable graceful stop
process.once('SIGINT', () => bot.stop('SIGINT'));
process.once('SIGTERM', () => bot.stop('SIGTERM'));
```

### Database Configuration

Check your `config/database.yml` for connection details:

```bash
cat config/database.mysql.yml
```

### Install Dependencies

```bash
npm install telegraf mysql2 uuid crc-32
npm install -D @types/node typescript ts-node
```

### package.json

```json
{
  "name": "fizzy-telegram-bot",
  "version": "1.0.0",
  "scripts": {
    "dev": "ts-node src/bot.ts",
    "build": "tsc",
    "start": "node dist/bot.js"
  },
  "dependencies": {
    "telegraf": "^4.15.0",
    "mysql2": "^3.6.5",
    "uuid": "^9.0.1",
    "crc-32": "^1.2.2",
    "dotenv": "^16.3.1"
  },
  "devDependencies": {
    "@types/node": "^20.10.0",
    "typescript": "^5.3.0",
    "ts-node": "^10.9.1"
  }
}
```

### .env

```env
BOT_TOKEN=your_telegram_bot_token
ACCOUNT_ID=your_account_uuid
USER_ID=your_user_uuid
BOARD_ID=your_default_board_uuid

DB_HOST=localhost
DB_USER=fizzy_user
DB_PASSWORD=your_password
DB_NAME=fizzy_production
```

---

## Option 2: JSON Endpoints (Limited, Cookie Auth)

Fizzy has **JSON views** for some resources, but they use **cookie-based authentication** which is challenging for bots.

### Available JSON Endpoints

All standard Rails routes support JSON format by adding `.json` extension:

**Cards:**
```http
GET /{account_id}/cards/{card_number}.json
```

Returns:
```json
{
  "id": "uuid",
  "title": "Card title",
  "status": "triaged",
  "golden": false,
  "last_active_at": "2025-12-07T10:30:00.000Z",
  "created_at": "2025-12-01T08:00:00.000Z",
  "url": "http://fizzy.localhost:3006/{account_id}/cards/123",
  "board": { "id": "uuid", "name": "Engineering" },
  "column": { "id": "uuid", "name": "In Progress", "color": "blue" },
  "creator": { "id": "uuid", "name": "Jane Smith" }
}
```

### The Problem: Cookie-Based Authentication

These endpoints require **session cookies**. **Not recommended** for production bots.

---

## Option 3: Webhooks (One-Way Push from Fizzy)

Fizzy has a **webhook system** that can push events to your bot. This is one-way: Fizzy → Your Bot.

### Available Webhook Events

Located in `app/models/webhook.rb:9-21`:

```
card_assigned, card_closed, card_postponed, card_auto_postponed,
card_board_changed, card_published, card_reopened,
card_sent_back_to_triage, card_triaged, card_unassigned,
comment_created
```

### Webhook Payload Format

```json
{
  "id": "event-uuid",
  "action": "card_assigned",
  "created_at": "2025-12-07T10:30:00.000Z",
  "eventable": {
    "id": "card-uuid",
    "title": "Fix authentication bug",
    "status": "triaged",
    "url": "http://fizzy.localhost:3006/12345678/cards/123",
    "board": { "id": "uuid", "name": "Engineering" }
  },
  "creator": { "id": "uuid", "name": "John Doe" }
}
```

### Setting Up Webhooks

**1. Create a webhook receiver endpoint:**

```typescript
import express from 'express';
import crypto from 'crypto';
import { Telegraf } from 'telegraf';

const app = express();
const bot = new Telegraf(process.env.BOT_TOKEN!);

const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET!;
const CHAT_ID = parseInt(process.env.CHAT_ID!);

interface WebhookEvent {
  id: string;
  action: string;
  created_at: string;
  eventable: any;
  creator: {
    id: string;
    name: string;
  };
  board: {
    id: string;
    name: string;
  };
}

// Middleware to verify HMAC signature
function verifySignature(req: express.Request, res: express.Response, next: express.NextFunction) {
  const signature = req.headers['x-fizzy-signature'] as string;
  const body = JSON.stringify(req.body);

  const expectedSignature = crypto
    .createHmac('sha256', WEBHOOK_SECRET)
    .update(body)
    .digest('hex');

  if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expectedSignature))) {
    return res.status(403).json({ error: 'Invalid signature' });
  }

  next();
}

app.use(express.json());

app.post('/fizzy-webhook', verifySignature, async (req, res) => {
  const event: WebhookEvent = req.body;

  console.log('Received webhook:', event.action);

  try {
    const message = formatWebhookMessage(event);

    if (message) {
      await bot.telegram.sendMessage(CHAT_ID, message, { parse_mode: 'Markdown' });
    }

    res.status(200).send('OK');
  } catch (error) {
    console.error('Error handling webhook:', error);
    res.status(500).send('Internal error');
  }
});

function formatWebhookMessage(event: WebhookEvent): string | null {
  switch (event.action) {
    case 'card_assigned':
      return (
        `🔔 *Card Assignment*\n\n` +
        `You were assigned to:\n` +
        `*${event.eventable.title}*\n\n` +
        `Board: ${event.board.name}\n` +
        `[View Card](${event.eventable.url})`
      );

    case 'comment_created':
      return (
        `💬 *New Comment*\n\n` +
        `${event.creator.name} commented:\n` +
        `_${event.eventable.body.plain_text}_\n\n` +
        `[View Comment](${event.eventable.url})`
      );

    case 'card_closed':
      return (
        `✅ *Card Closed*\n\n` +
        `${event.creator.name} closed:\n` +
        `*${event.eventable.title}*`
      );

    case 'card_published':
      return (
        `📋 *New Card Published*\n\n` +
        `${event.creator.name} published:\n` +
        `*${event.eventable.title}*\n\n` +
        `Board: ${event.board.name}\n` +
        `[View Card](${event.eventable.url})`
      );

    case 'card_postponed':
      return (
        `⏸️ *Card Postponed*\n\n` +
        `${event.creator.name} postponed:\n` +
        `*${event.eventable.title}*`
      );

    default:
      console.log('Unhandled webhook action:', event.action);
      return null;
  }
}

const PORT = process.env.PORT || 3000;

app.listen(PORT, () => {
  console.log(`Webhook server listening on port ${PORT}`);
});

bot.launch();

// Graceful shutdown
process.once('SIGINT', () => {
  bot.stop('SIGINT');
  process.exit(0);
});

process.once('SIGTERM', () => {
  bot.stop('SIGTERM');
  process.exit(0);
});
```

**2. Install dependencies:**

```bash
npm install express telegraf
npm install -D @types/express
```

**3. Configure webhook in Fizzy UI:**
- Navigate to `/{account_id}/boards/{board_id}/webhooks`
- Create new webhook
- Set URL to your bot's public endpoint
- Select which events to subscribe to
- Save the signing secret for verification

**4. Expose your bot** via ngrok or deploy:
```bash
ngrok http 3000
# Use the HTTPS URL in Fizzy webhook config
```

### Webhook Limitations
- **One-way only**: Fizzy pushes to you, you can't query back
- **Board-scoped**: Each webhook is tied to a single board
- **Auto-disabled on failures**: After 10 consecutive failures in 1 hour

---

## Option 4: Build Custom API Endpoints (Most Flexible)

Add token-based API authentication to Fizzy for full two-way integration.

### Quick Implementation

**1. Add API Token Model**

Create migration:
```bash
bin/rails generate migration CreateApiTokens user:uuid token:string:uniq last_used_at:datetime
```

Edit migration:
```ruby
class CreateApiTokens < ActiveRecord::Migration[8.2]
  def change
    create_table :api_tokens, id: :uuid do |t|
      t.uuid :user_id, null: false
      t.uuid :account_id, null: false
      t.string :token, null: false
      t.string :name
      t.datetime :last_used_at
      t.timestamps

      t.index [:user_id]
      t.index [:token], unique: true
      t.index [:account_id]
    end
  end
end
```

Run migration:
```bash
bin/rails db:migrate
```

**2. Create API Token Model**

`app/models/api_token.rb`:
```ruby
class ApiToken < ApplicationRecord
  belongs_to :user
  belongs_to :account, default: -> { user.account }

  has_secure_token :token, length: 32

  before_validation :set_account

  def use!
    update_column(:last_used_at, Time.current)
  end

  private
    def set_account
      self.account ||= user.account
    end
end
```

Update `app/models/user.rb`:
```ruby
has_many :api_tokens, dependent: :destroy
```

**3. Create API Authentication**

`app/controllers/concerns/api_authentication.rb`:
```ruby
module ApiAuthentication
  extend ActiveSupport::Concern

  included do
    skip_before_action :require_authentication
    skip_before_action :require_account
    before_action :authenticate_with_token!
  end

  private
    def authenticate_with_token!
      token = request.headers['Authorization']&.remove('Bearer ')

      if token.blank?
        render json: { error: 'Missing authorization token' }, status: :unauthorized
        return
      end

      api_token = ApiToken.find_by(token: token)

      if api_token.nil?
        render json: { error: 'Invalid token' }, status: :unauthorized
        return
      end

      api_token.use!
      Current.session = api_token.user.identity.sessions.first_or_create!(
        user_agent: 'API',
        ip_address: request.remote_ip
      )
      Current.account = api_token.account
    end
end
```

**4. Create API Controllers**

`app/controllers/api/base_controller.rb`:
```ruby
class Api::BaseController < ActionController::API
  include ApiAuthentication
  include CurrentRequest

  rescue_from ActiveRecord::RecordNotFound, with: :not_found
  rescue_from ActiveRecord::RecordInvalid, with: :unprocessable_entity

  private
    def not_found
      render json: { error: 'Not found' }, status: :not_found
    end

    def unprocessable_entity(exception)
      render json: { error: exception.message }, status: :unprocessable_entity
    end
end
```

`app/controllers/api/cards_controller.rb`:
```ruby
class Api::CardsController < Api::BaseController
  def index
    cards = Current.user.accessible_cards
      .where.not(status: ['closed', 'not_now'])
      .includes(:board, :column, :creator, :assignees)
      .order(last_active_at: :desc)
      .limit(params[:limit] || 50)

    render json: cards.map { |card|
      {
        id: card.id,
        number: card.number,
        title: card.title,
        status: card.status,
        due_on: card.due_on,
        board: { id: card.board.id, name: card.board.name },
        column: card.column ? { id: card.column.id, name: card.column.name } : nil,
        url: card_url(card)
      }
    }
  end

  def show
    card = Current.user.accessible_cards.find_by!(number: params[:id])
    render json: {
      id: card.id,
      number: card.number,
      title: card.title,
      status: card.status,
      description: card.description.to_plain_text,
      board: { id: card.board.id, name: card.board.name }
    }
  end

  def create
    board = Current.user.boards.find(params[:board_id])
    card = board.cards.create!(card_params.merge(creator: Current.user))
    render json: { id: card.id, number: card.number, title: card.title },
           status: :created
  end

  def update
    card = Current.user.accessible_cards.find_by!(number: params[:id])
    card.update!(card_params)
    render json: { id: card.id, number: card.number, title: card.title }
  end

  private
    def card_params
      params.require(:card).permit(:title, :description, :status, :due_on)
    end
end
```

**5. Add API Routes**

`config/routes.rb`:
```ruby
namespace :api do
  namespace :v1 do
    resources :cards, only: [:index, :show, :create, :update] do
      resources :comments, only: [:index, :create]
    end
    resources :boards, only: [:index, :show]
  end
end
```

**6. Generate Token**

```bash
bin/rails console
```

```ruby
user = User.find_by(email: 'your@email.com')
token = user.api_tokens.create!(name: 'Telegram Bot')
puts "Token: #{token.token}"
# Save this securely!
```

**7. Use in TypeScript Bot**

```typescript
import axios, { AxiosInstance } from 'axios';

interface Card {
  id: string;
  number: number;
  title: string;
  status: string;
  due_on?: string;
  board: {
    id: string;
    name: string;
  };
  column?: {
    id: string;
    name: string;
  };
  url: string;
}

interface CreateCardParams {
  title: string;
  description?: string;
  status?: string;
  due_on?: string;
}

class FizzyAPI {
  private client: AxiosInstance;

  constructor(baseURL: string, apiToken: string) {
    this.client = axios.create({
      baseURL,
      headers: {
        Authorization: `Bearer ${apiToken}`,
        'Content-Type': 'application/json',
      },
    });
  }

  async getCards(limit: number = 50): Promise<Card[]> {
    const response = await this.client.get<Card[]>('/api/v1/cards', {
      params: { limit },
    });
    return response.data;
  }

  async getCard(cardNumber: number): Promise<Card> {
    const response = await this.client.get<Card>(`/api/v1/cards/${cardNumber}`);
    return response.data;
  }

  async createCard(boardId: string, params: CreateCardParams): Promise<Card> {
    const response = await this.client.post<Card>('/api/v1/cards', {
      board_id: boardId,
      card: params,
    });
    return response.data;
  }

  async updateCard(cardNumber: number, params: Partial<CreateCardParams>): Promise<Card> {
    const response = await this.client.patch<Card>(`/api/v1/cards/${cardNumber}`, {
      card: params,
    });
    return response.data;
  }

  async addComment(cardNumber: number, content: string): Promise<void> {
    await this.client.post(`/api/v1/cards/${cardNumber}/comments`, {
      comment: { content },
    });
  }
}

// Usage in bot
import { Telegraf } from 'telegraf';

const bot = new Telegraf(process.env.BOT_TOKEN!);
const fizzyAPI = new FizzyAPI(
  process.env.FIZZY_URL!,
  process.env.FIZZY_API_TOKEN!
);

bot.command('my_cards', async (ctx) => {
  try {
    const cards = await fizzyAPI.getCards(10);

    if (cards.length === 0) {
      return ctx.reply('You have no active cards.');
    }

    const message = cards
      .map(
        (card) =>
          `#${card.number} - ${card.title}\n` +
          `   📋 ${card.board.name} | ${card.status}` +
          (card.column ? `\n   📍 ${card.column.name}` : '') +
          (card.due_on ? `\n   📅 Due: ${card.due_on}` : '')
      )
      .join('\n\n');

    ctx.reply(`📋 Your Cards:\n\n${message}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error fetching cards.');
  }
});

bot.command('card', async (ctx) => {
  const cardNumber = parseInt(ctx.message.text.split(' ')[1]);

  if (!cardNumber) {
    return ctx.reply('Usage: /card <number>');
  }

  try {
    const card = await fizzyAPI.getCard(cardNumber);

    const message = [
      `📋 Card #${card.number}`,
      `\n**${card.title}**`,
      `\nBoard: ${card.board.name}`,
      `Status: ${card.status}`,
      card.column ? `Column: ${card.column.name}` : '',
      card.due_on ? `Due: ${card.due_on}` : '',
      `\n[View in Fizzy](${card.url})`,
    ]
      .filter(Boolean)
      .join('\n');

    ctx.reply(message, { parse_mode: 'Markdown' });
  } catch (error) {
    console.error(error);
    ctx.reply('Card not found.');
  }
});

bot.command('create', async (ctx) => {
  const title = ctx.message.text.split(' ').slice(1).join(' ');

  if (!title) {
    return ctx.reply('Usage: /create <card title>');
  }

  try {
    const card = await fizzyAPI.createCard(process.env.DEFAULT_BOARD_ID!, { title });

    ctx.reply(`✅ Created card #${card.number}: ${card.title}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error creating card.');
  }
});

bot.command('comment', async (ctx) => {
  const args = ctx.message.text.split(' ').slice(1);
  const cardNumber = parseInt(args[0]);
  const content = args.slice(1).join(' ');

  if (!cardNumber || !content) {
    return ctx.reply('Usage: /comment <card_number> <your comment>');
  }

  try {
    await fizzyAPI.addComment(cardNumber, content);
    ctx.reply(`✅ Comment added to card #${cardNumber}`);
  } catch (error) {
    console.error(error);
    ctx.reply('Error adding comment.');
  }
});

bot.launch();

process.once('SIGINT', () => bot.stop('SIGINT'));
process.once('SIGTERM', () => bot.stop('SIGTERM'));
```

**Install dependencies:**

```bash
npm install axios telegraf dotenv
npm install -D @types/node typescript ts-node
```

---

## Recommendation Matrix

| Approach | Complexity | Two-Way? | Real-time? | Best For |
|----------|-----------|----------|------------|----------|
| **Direct DB Access** | Low | ✅ Yes | ✅ Yes | Self-hosted, full control |
| **JSON Endpoints** | Medium | ✅ Yes | ✅ Yes | Quick prototype (cookie auth is painful) |
| **Webhooks** | Low | ❌ One-way | ✅ Yes | Notifications only |
| **Custom API** | High | ✅ Yes | ✅ Yes | Production-grade integration |

---

## Recommended Approach: Hybrid

For a **self-hosted Fizzy instance** with a **Telegram bot**:

### Best Combination

1. **Direct database access** for reading data
   - Fast, simple, no auth needed
   - Perfect for queries

2. **Webhooks** for real-time notifications
   - Get notified about assignments, comments
   - Push updates to Telegram

3. **Direct DB writes** OR **Custom API** for modifications
   - If comfortable with SQL: direct writes
   - If need validation: build minimal API

### Example Bot Commands

```
/my_cards - Show your assigned cards
/board <name> - Show cards in a board
/search <query> - Search cards
/card <number> - Show card details
/create <title> - Create new card
/comment <number> <text> - Add comment
/close <number> - Close card
```

Webhooks handle notifications automatically!

---

## Complete Project Structure

```
fizzy-telegram-bot/
├── src/
│   ├── bot.ts              # Main bot file
│   ├── db.ts               # Database connection
│   ├── api.ts              # API client
│   ├── webhooks.ts         # Webhook server
│   └── types.ts            # TypeScript types
├── package.json
├── tsconfig.json
├── .env
└── README.md
```

### tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "moduleResolution": "node"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

---

## Security Considerations

### Direct Database Access
- Use read-only credentials for query-only operations
- Create dedicated bot user with limited permissions
- Always scope by `account_id`
- Validate user permissions before writes

### Webhooks
- **Always verify HMAC signature**
- Use HTTPS endpoints only
- Implement rate limiting

### Custom API
- Use HTTPS only
- Rotate tokens regularly
- Implement rate limiting
- Log all API access

---

## Useful Files Reference

- `DATABASE_SCHEMA.md` - Complete database documentation
- `config/routes.rb` - All existing routes
- `app/models/webhook.rb` - Webhook event types
- `app/views/**/*.json.jbuilder` - Existing JSON serializers
