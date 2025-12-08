# Fizzy Database Schema Documentation

This document describes the complete database schema for Fizzy, designed to help you build integrations like Telegram bots.

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Core Tables](#core-tables)
- [Authentication & Users](#authentication--users)
- [Boards & Cards](#boards--cards)
- [Card Features](#card-features)
- [Events & Notifications](#events--notifications)
- [Search System](#search-system)
- [Webhooks](#webhooks)
- [Supporting Tables](#supporting-tables)

---

## Architecture Overview

### Multi-Tenancy (URL-Based)
Fizzy uses **URL path-based multi-tenancy**:
- Each Account has a unique `external_account_id` (7+ digits)
- URLs are prefixed: `/{account_id}/boards/...`
- **CRITICAL**: All models include `account_id` for data isolation
- When building a bot, you must always scope queries by `account_id`

### UUID Primary Keys
- All tables use UUIDs (UUIDv7 format, base36-encoded as 25-char strings)
- All ID fields are of type `uuid`

---

## Core Tables

### accounts
The top-level tenant/organization.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| name | string | Account name |
| external_account_id | bigint | Unique 7+ digit ID for URL routing (unique) |
| cards_count | bigint | Counter cache for card numbering |
| created_at | datetime | |
| updated_at | datetime | |

**Key relationships:**
- `has_many :users`
- `has_many :boards`
- `has_many :cards`
- `has_many :tags`
- `has_many :columns`
- `has_many :webhooks`
- `has_one :join_code` (for team invitations)

---

## Authentication & Users

### identities
Global user identity (email-based), can exist in multiple accounts.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| email_address | string | Unique email (unique index) |
| staff | boolean | Is staff member (default: false) |
| created_at | datetime | |
| updated_at | datetime | |

**Key relationships:**
- `has_many :users` (across multiple accounts)
- `has_many :sessions`
- `has_many :magic_links`
- `has_one_attached :avatar`

### users
Account-specific user membership.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| identity_id | uuid | Foreign key to identities (nullable, null if deactivated) |
| name | string | Display name |
| role | string | Role: owner/admin/member/system (default: member) |
| active | boolean | Is user active (default: true) |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `account_id + identity_id`

**Key relationships:**
- `belongs_to :account`
- `belongs_to :identity`
- `has_many :accesses` (board access)
- `has_many :assignments` (as assignee)
- `has_many :comments`
- `has_many :pins`
- `has_many :filters`

### sessions
Active login sessions.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| identity_id | uuid | Foreign key to identities |
| ip_address | string | Session IP |
| user_agent | string(4096) | Browser/client info |
| created_at | datetime | |
| updated_at | datetime | |

### magic_links
Passwordless authentication codes.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| identity_id | uuid | Foreign key to identities |
| code | string | Unique magic link code (unique) |
| purpose | integer | Purpose enum |
| expires_at | datetime | Expiration timestamp |
| created_at | datetime | |
| updated_at | datetime | |

---

## Boards & Cards

### boards
Primary organizational unit for cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| creator_id | uuid | Foreign key to users |
| name | string | Board name |
| all_access | boolean | If true, all users can access (default: false) |
| created_at | datetime | |
| updated_at | datetime | |

**Key relationships:**
- `belongs_to :account`
- `belongs_to :creator` (User)
- `has_many :cards`
- `has_many :columns`
- `has_many :accesses` (board access permissions)
- `has_many :events`
- `has_many :webhooks`
- `has_one :publication` (for public sharing)
- `has_one :entropy` (auto-postpone settings)

### accesses
Board access permissions (only used when `board.all_access = false`).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| board_id | uuid | Foreign key to boards |
| user_id | uuid | Foreign key to users |
| involvement | string | Level: access_only (default) |
| accessed_at | datetime | Last access time |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `board_id + user_id`

### columns
Workflow stages within a board (kanban columns).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| board_id | uuid | Foreign key to boards |
| name | string | Column name |
| color | string | Color code |
| position | integer | Display order (default: 0) |
| created_at | datetime | |
| updated_at | datetime | |

### cards
The main work item (task/issue).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| board_id | uuid | Foreign key to boards |
| column_id | uuid | Foreign key to columns (nullable) |
| creator_id | uuid | Foreign key to users |
| number | bigint | Sequential number within account (unique per account) |
| title | string | Card title |
| status | string | drafted/triaged/closed/not_now (default: drafted) |
| due_on | date | Due date (nullable) |
| last_active_at | datetime | Last activity timestamp |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `account_id + number`

**Key relationships:**
- `belongs_to :account`
- `belongs_to :board`
- `belongs_to :column`
- `belongs_to :creator` (User)
- `has_many :comments`
- `has_many :assignments` (assigned users)
- `has_many :taggings` / `has_many :tags`
- `has_many :steps` (checklist items)
- `has_one :closure` (when closed)
- `has_one :not_now` (when postponed)
- `has_one :goldness` (when marked golden/important)
- `has_one :activity_spike` (when recently active)
- `has_rich_text :description`
- `has_one_attached :image`

**Card statuses:**
- `drafted`: Initial state, in triage
- `triaged`: Published to a column
- `closed`: Completed/resolved
- `not_now`: Postponed for later

---

## Card Features

### assignments
Users assigned to cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| assignee_id | uuid | Foreign key to users (who is assigned) |
| assigner_id | uuid | Foreign key to users (who assigned) |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `assignee_id + card_id`

### comments
Comments on cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| creator_id | uuid | Foreign key to users |
| created_at | datetime | |
| updated_at | datetime | |

**Key relationships:**
- `has_rich_text :content`
- `has_many :reactions`
- `has_many :mentions`

### reactions
Emoji reactions to comments.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| comment_id | uuid | Foreign key to comments |
| reacter_id | uuid | Foreign key to users |
| content | string(16) | Emoji content |
| created_at | datetime | |
| updated_at | datetime | |

### taggings
Links cards to tags.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| tag_id | uuid | Foreign key to tags |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `card_id + tag_id`

### tags
Labels for categorizing cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| title | string | Tag name |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `account_id + title`

### steps
Checklist items within cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| content | text | Step description |
| completed | boolean | Is completed (default: false) |
| created_at | datetime | |
| updated_at | datetime | |

### closures
Tracks closed cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| user_id | uuid | Foreign key to users (who closed it, nullable) |
| created_at | datetime | When closed |
| updated_at | datetime | |

**Unique constraint:** `card_id` (one closure per card)

### card_not_nows
Tracks postponed cards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| user_id | uuid | Foreign key to users (who postponed, nullable) |
| created_at | datetime | When postponed |
| updated_at | datetime | |

**Unique constraint:** `card_id`

### card_goldnesses
Marks cards as "golden" (important/prioritized).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `card_id`

### card_activity_spikes
Tracks cards with recent activity bursts.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `card_id`

### pins
User-specific card pins.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| user_id | uuid | Foreign key to users |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `card_id + user_id`

### watches
User card watching preferences.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards |
| user_id | uuid | Foreign key to users |
| watching | boolean | Is watching (default: true) |
| created_at | datetime | |
| updated_at | datetime | |

### mentions
User mentions in cards/comments.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| source_id | uuid | Polymorphic: card or comment ID |
| source_type | string | Polymorphic: Card or Comment |
| mentionee_id | uuid | Foreign key to users (who was mentioned) |
| mentioner_id | uuid | Foreign key to users (who mentioned) |
| created_at | datetime | |
| updated_at | datetime | |

---

## Events & Notifications

### events
Records all significant actions for activity timeline.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| board_id | uuid | Foreign key to boards |
| eventable_id | uuid | Polymorphic: related object ID |
| eventable_type | string | Polymorphic: object type |
| creator_id | uuid | Foreign key to users |
| action | string | Event type (e.g., card_created, comment_added) |
| particulars | json | Action-specific data (default: {}) |
| created_at | datetime | |
| updated_at | datetime | |

**Common event actions:**
- `card_created`, `card_updated`, `card_closed`, `card_postponed`
- `comment_added`, `comment_updated`
- `assignment_added`, `assignment_removed`
- `tag_added`, `tag_removed`
- `board_changed`, `column_changed`
- Many more...

### notifications
User-specific notifications.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| user_id | uuid | Foreign key to users (recipient) |
| creator_id | uuid | Foreign key to users (who triggered, nullable) |
| source_id | uuid | Polymorphic: source object ID |
| source_type | string | Polymorphic: source type |
| read_at | datetime | When read (nullable) |
| created_at | datetime | |
| updated_at | datetime | |

### notification_bundles
Groups notifications for batch email delivery.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| user_id | uuid | Foreign key to users |
| status | integer | Delivery status enum (default: 0) |
| starts_at | datetime | Bundle period start |
| ends_at | datetime | Bundle period end |
| created_at | datetime | |
| updated_at | datetime | |

### push_subscriptions
Web push notification subscriptions.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| user_id | uuid | Foreign key to users |
| endpoint | text | Push service endpoint |
| p256dh_key | string | Encryption key |
| auth_key | string | Auth key |
| user_agent | string(4096) | Client info |
| created_at | datetime | |
| updated_at | datetime | |

---

## Search System

Fizzy uses **16-shard MySQL full-text search** instead of Elasticsearch. Search records are denormalized for performance.

### search_records_0 through search_records_15
16 sharded tables for full-text search (shard determined by `CRC32(account_id) % 16`).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| account_key | string | Account key for search scoping |
| searchable_id | uuid | Polymorphic: searchable object ID |
| searchable_type | string | Polymorphic: Card or Comment |
| board_id | uuid | Foreign key to boards |
| card_id | uuid | Foreign key to cards |
| title | string | Searchable title |
| content | text | Searchable content |
| created_at | datetime | |

**Unique constraint:** `searchable_type + searchable_id`

**Full-text index:** `account_key + content + title`

### search_queries
User search history.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| user_id | uuid | Foreign key to users |
| terms | string(2000) | Search terms |
| created_at | datetime | |
| updated_at | datetime | |

---

## Entropy System

Cards automatically "postpone" (move to "not now") after inactivity to prevent endless todo lists.

### entropies
Entropy (auto-postpone) configuration.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| container_id | uuid | Polymorphic: Account or Board ID |
| container_type | string | Polymorphic: Account or Board |
| auto_postpone_period | bigint | Seconds of inactivity before auto-postpone (default: 2592000 = 30 days) |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `container_type + container_id`

---

## Webhooks

### webhooks
Webhook configurations for external integrations.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| board_id | uuid | Foreign key to boards |
| name | string | Webhook name |
| url | text | Webhook endpoint URL |
| signing_secret | string | HMAC signing secret |
| subscribed_actions | text | JSON array of subscribed event actions |
| active | boolean | Is active (default: true) |
| created_at | datetime | |
| updated_at | datetime | |

### webhook_deliveries
Webhook delivery attempts and results.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| webhook_id | uuid | Foreign key to webhooks |
| event_id | uuid | Foreign key to events |
| state | string | Delivery state (pending/success/failed) |
| request | text | Request payload |
| response | text | Response received |
| created_at | datetime | |
| updated_at | datetime | |

### webhook_delinquency_trackers
Tracks webhook failures for automatic disabling.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| webhook_id | uuid | Foreign key to webhooks |
| consecutive_failures_count | integer | Failure count (default: 0) |
| first_failure_at | datetime | First failure timestamp |
| created_at | datetime | |
| updated_at | datetime | |

---

## Supporting Tables

### board_publications
Public sharing of boards.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| board_id | uuid | Foreign key to boards |
| key | string | Shareable public key |
| created_at | datetime | |
| updated_at | datetime | |

### account_join_codes
Invite codes for joining accounts.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| code | string | Join code |
| usage_limit | bigint | Max uses (default: 10) |
| usage_count | bigint | Current uses (default: 0) |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `account_id + code`

### account_exports
Account data export jobs.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| user_id | uuid | Foreign key to users (who requested) |
| status | string | pending/completed (default: pending) |
| completed_at | datetime | Completion timestamp |
| created_at | datetime | |
| updated_at | datetime | |

### filters
Saved filter/search configurations.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| creator_id | uuid | Foreign key to users |
| fields | json | Filter field values (default: {}) |
| params_digest | string | Hash of filter params |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `creator_id + params_digest`

**Many-to-many filter relationships:**
- `filters_tags` - filter → tags
- `boards_filters` - filter → boards
- `assignees_filters` - filter → assignees
- `assigners_filters` - filter → assigners
- `creators_filters` - filter → creators
- `closers_filters` - filter → closers

### user_settings
Per-user preferences.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| user_id | uuid | Foreign key to users |
| timezone_name | string | User timezone |
| bundle_email_frequency | integer | Email bundle frequency enum (default: 0) |
| created_at | datetime | |
| updated_at | datetime | |

### card_engagements
Tracks user engagement with cards (for "doing" status).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| card_id | uuid | Foreign key to cards (nullable) |
| status | string | doing/done (default: doing) |
| created_at | datetime | |
| updated_at | datetime | |

### Active Storage Tables

#### active_storage_blobs
File blob metadata.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| key | string | Unique storage key |
| filename | string | Original filename |
| content_type | string | MIME type |
| byte_size | bigint | File size |
| checksum | string | File checksum |
| service_name | string | Storage service |
| metadata | text | Additional metadata |
| created_at | datetime | |

#### active_storage_attachments
Links blobs to records.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| name | string | Attachment name |
| record_id | uuid | Polymorphic: owner ID |
| record_type | string | Polymorphic: owner type |
| blob_id | uuid | Foreign key to blobs |
| created_at | datetime | |

**Unique constraint:** `record_type + record_id + name + blob_id`

#### active_storage_variant_records
Image variants (thumbnails, etc.).

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| blob_id | uuid | Foreign key to blobs |
| variation_digest | string | Variant configuration hash |

### Action Text Tables

#### action_text_rich_texts
Rich text content storage.

| Column | Type | Description |
|--------|------|-------------|
| id | uuid | Primary key |
| account_id | uuid | Foreign key to accounts |
| name | string | Field name (e.g., "description", "content") |
| body | text(long) | HTML content |
| record_id | uuid | Polymorphic: owner ID |
| record_type | string | Polymorphic: owner type |
| created_at | datetime | |
| updated_at | datetime | |

**Unique constraint:** `record_type + record_id + name`

---

## Key Concepts for Bot Development

### Data Isolation
**ALWAYS scope queries by `account_id`** to ensure proper multi-tenancy:
```ruby
# Good
account.cards.where(status: 'triaged')

# Bad - crosses account boundaries!
Card.where(status: 'triaged')
```

### Card Lifecycle
1. **Created** → `status: 'drafted'`, in triage area
2. **Published** → `status: 'triaged'`, assigned to a column
3. **Closed** → `status: 'closed'`, has closure record
4. **Postponed** → `status: 'not_now'`, has not_now record

### Card Numbers
- Sequential per account (not globally)
- Use `card.number` for display: `#123`
- Use `account.increment!(:cards_count)` to assign new numbers

### Board Access
- If `board.all_access = true`, all account users can access
- If `board.all_access = false`, only users with `accesses` records can access
- Check with `board.accessible_to?(user)`

### Events Drive Everything
- All significant actions create `Event` records
- Events trigger notifications, webhooks, activity timeline
- Use events to track card history

### Rich Text
- Card descriptions and comment content use Action Text
- Stored in `action_text_rich_texts` table
- Contains HTML with embedded attachments/mentions

---

## Bot Integration Examples

### List User's Cards
```ruby
account = Account.find_by(external_account_id: account_id)
user = account.users.find_by(identity: identity)

# Cards assigned to user
cards = account.cards
  .joins(:assignments)
  .where(assignments: { assignee_id: user.id })
  .where.not(status: ['closed', 'not_now'])
  .order(last_active_at: :desc)
```

### Create a Card
```ruby
card = account.cards.create!(
  board: board,
  creator: user,
  title: "New task from Telegram",
  status: 'drafted'
)

# Publish to column
card.update!(column: board.columns.first, status: 'triaged')
```

### Add Comment
```ruby
comment = card.comments.create!(
  creator: user,
  content: "Comment from Telegram bot"
)
```

### Close Card
```ruby
card.update!(status: 'closed')
card.create_closure!(user: user)
```

### Search Cards
```ruby
# Use the Search model which handles sharding
Search.search(account, query: "bug fix", scope: :cards)
```

---

## Important Notes

1. **All timestamps are UTC** - convert to user's timezone from `user_settings.timezone_name`
2. **Card numbers are per-account** - don't use them as global IDs
3. **Always check board access** before showing cards to users
4. **Events are the source of truth** for activity - use them for history
5. **Rich text content** may contain HTML - sanitize for Telegram
6. **Webhooks use HMAC signatures** - verify before processing
7. **Search is sharded** - use the Search model, not direct queries
8. **Auto-postpone runs hourly** - cards can change status automatically
