# VentureMate Backend

A production-ready Rust backend for VentureMate - an AI-powered platform for launching businesses.

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+
- PostgreSQL (via Supabase)
- Anthropic API key (for Claude AI)

### 1. Clone & Setup

```bash
git clone <repo-url>
cd bcknd
cp .env.example .env
```

### 2. Supabase Setup

1. **Create a Supabase Project**:
   - Go to [supabase.com](https://supabase.com) and create a new project
   - Wait for the database to be provisioned

2. **Get Connection Details**:
   - Go to **Settings** → **Database**
   - Copy the **Connection String** (URI format)
   - It looks like: `postgresql://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:5432/postgres`

3. **Get API Keys**:
   - Go to **Settings** → **API**
   - Copy **Project URL** (`NEXT_PUBLIC_SUPABASE_URL`)
   - Copy **anon/public** key (`NEXT_PUBLIC_SUPABASE_ANON_KEY`)
   - Copy **service_role/secret** key (`SUPABASE_SERVICE_ROLE_KEY`)

4. **Configure Environment**:
   ```bash
   # Edit .env with your Supabase credentials
   SUPABASE_DB_URL=postgresql://postgres.your-project:your-password@aws-0-your-region.pooler.supabase.com:5432/postgres
   NEXT_PUBLIC_SUPABASE_URL=https://your-project.supabase.co
   NEXT_PUBLIC_SUPABASE_ANON_KEY=your-anon-key
   SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
   JWT_SECRET=your-super-secret-key
   ANTHROPIC_API_KEY=sk-ant-api03-your-key
   ```

### 3. Database Migrations

Migrations run automatically on startup. The migration file is at `migrations/20240320000000_initial_schema.sql`.

To run migrations manually via Supabase:
1. Go to Supabase Dashboard → SQL Editor
2. Copy contents of `migrations/20240320000000_initial_schema.sql`
3. Run the SQL

### 4. Run the Server

```bash
# Development
cargo run

# With logging
RUST_LOG=debug cargo run

# Production build
SQLX_OFFLINE=true cargo build --release
```

The server will start at `http://127.0.0.1:8080`

## 📊 API Documentation

### Health Check
```bash
curl http://localhost:8080/health
```

### Authentication

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/auth/register` | POST | Create new account |
| `/auth/login` | POST | Login with credentials |
| `/auth/refresh` | POST | Refresh access token |
| `/auth/logout` | POST | Logout user |
| `/auth/forgot-password` | POST | Request password reset |
| `/auth/reset-password` | POST | Reset password with token |

### Users

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/users/me` | GET | Get current user |
| `/users/me` | PUT | Update user profile |
| `/users/me` | DELETE | Delete account |

### Onboarding

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/onboarding/start` | POST | Start onboarding session |
| `/onboarding/idea` | POST | Submit business idea |
| `/onboarding/founder` | POST | Submit founder profile |
| `/onboarding/business` | POST | Submit business details |
| `/onboarding/complete` | POST | Complete onboarding |
| `/onboarding/status` | GET | Get onboarding status |

### Businesses

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/businesses` | POST | Create business |
| `/businesses` | GET | List user's businesses |
| `/businesses/:id` | GET | Get business details |
| `/businesses/:id` | PUT | Update business |
| `/businesses/:id` | DELETE | Delete business |
| `/businesses/:id/checklist` | GET | Get compliance checklist |
| `/businesses/:id/checklist/:item` | PATCH | Update checklist item |

### AI Generation (Claude)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/ai/business-plan` | POST | Generate business plan |
| `/ai/pitch-deck` | POST | Generate pitch deck |
| `/ai/name-ideas` | POST | Generate business names |
| `/ai/tagline` | POST | Generate taglines |
| `/ai/logo` | POST | Generate logo concepts |
| `/ai/logo/regenerate` | POST | Regenerate logo variations |
| `/ai/logo/select` | POST | Select final logo |

### Subscriptions

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/subscriptions/plans` | GET | List available plans |
| `/subscriptions/current` | GET | Get current subscription |
| `/subscriptions` | POST | Create subscription |
| `/subscriptions` | DELETE | Cancel subscription |
| `/subscriptions/invoices` | GET | Get billing history |

## 🗄️ Database Schema

### Core Tables
- `users` - User accounts
- `user_sessions` - JWT session tracking
- `password_reset_tokens` - Password reset flow
- `email_verifications` - Email verification tokens

### Business Tables
- `businesses` - Business entities
- `business_members` - Team members
- `industries` - Industry classifications

### AI Tables
- `ai_generation_jobs` - AI job tracking
- `generated_documents` - AI-generated content

### Subscription Tables
- `subscriptions` - User subscriptions
- `subscription_plans` - Available plans (Free, Starter, Professional, Enterprise)
- `invoices` - Billing records
- `payment_methods` - Saved payment methods

### Additional Tables
- `uploaded_files`, `documents`
- `marketplace_listings`, `marketplace_inquiries`
- `projects`, `project_tasks`
- `websites`, `website_pages`
- `onboarding_sessions`, `checklist_categories`, `checklist_items`
- `notification_preferences`, `audit_logs`

## 🔧 Configuration

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `SUPABASE_DB_URL` | ✅ | PostgreSQL connection string |
| `NEXT_PUBLIC_SUPABASE_URL` | ✅ | Supabase project URL |
| `NEXT_PUBLIC_SUPABASE_ANON_KEY` | ✅ | Supabase anon key |
| `SUPABASE_SERVICE_ROLE_KEY` | ✅ | Supabase service role key |
| `JWT_SECRET` | ✅ | Secret for JWT signing |
| `ANTHROPIC_API_KEY` | ✅ | Claude API key |
| `STRIPE_SECRET_KEY` | ❌ | Stripe secret (payments) |
| `HOST` | ❌ | Server host (default: 127.0.0.1) |
| `PORT` | ❌ | Server port (default: 8080) |

### Supabase Connection Pooler

The backend automatically detects and optimizes for Supabase's connection pooler:

- **Pooled Connection** (port 6543): Recommended for serverless, uses smaller connection pool
- **Direct Connection** (port 5432): For long-running connections, larger pool

## 🧪 Testing

```bash
# Run tests
cargo test

# Run with environment
cargo test -- --test-threads=1
```

## 📦 Building for Production

```bash
# Build release binary
SQLX_OFFLINE=true cargo build --release

# Binary location: target/release/bcknd
```

## 🐳 Docker (Optional)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN SQLX_OFFLINE=true cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/bcknd /usr/local/bin/
CMD ["bcknd"]
```

## 🔒 Security Features

- Argon2 password hashing
- JWT authentication with refresh tokens
- Row-level security ready (RLS policies can be added in Supabase)
- CORS configuration
- Input validation with `validator`
- SQL injection protection via SQLx prepared statements

## 📝 License

MIT

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request
