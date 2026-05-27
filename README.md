# 🌿 RedLeaf CMS

A lightweight, blazing-fast CMS built with Rust — aiming for WordPress parity without the bloat.

> 🇯🇵 Japanese documentation: [README_ja.md](README_ja.md)

## 🚀 About

**RedLeaf** is a modern CMS powered by Rust.  
It combines the **stability of systems programming** with the **flexibility of web publishing**.

- ⚡ **Fast** — compiled Rust backend, minimal runtime overhead
- 🪶 **Lightweight** — single binary, zero runtime dependencies
- 🔒 **Secure** — Argon2id password hashing, JWT authentication
- 🌐 **Headless Ready** — full REST API included
- 🐳 **Docker Ready** — multi-stage build, production-grade image

## 🔧 Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (stable) |
| Web Framework | [Axum](https://github.com/tokio-rs/axum) 0.8 |
| Database | SQLite via [SQLx](https://github.com/launchbadge/sqlx) 0.8 |
| Templates | [Askama](https://github.com/djc/askama) 0.14 (compiled) |
| Auth | JWT ([jsonwebtoken](https://github.com/Keats/jsonwebtoken)) + Argon2id |
| Search | SQLite FTS5 (full-text search) |

## ⚙️ Quick Start

### Development

```bash
git clone https://github.com/yourname/redleaf.git
cd redleaf
cp .env.example .env   # edit JWT_SECRET before production use
cargo run
```

Open http://localhost:3000

First-time setup: navigate to http://localhost:3000/setup to create your admin account.

### Docker

```bash
docker build -t redleaf .
docker run -p 3000:3000 \
  -v redleaf-data:/app/data \
  -v redleaf-uploads:/app/static/uploads \
  -e JWT_SECRET=your-secret-here \
  redleaf
```

## 🗂️ Directory Structure

```
redleaf/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Router setup
│   ├── auth.rs              # JWT generation & validation
│   ├── db.rs                # SQLite connection pool
│   ├── errors.rs            # Unified error types
│   ├── middleware.rs        # Auth middleware
│   ├── util.rs              # slugify / render / FTS utilities
│   ├── models/
│   │   ├── post.rs          # Post model (CRUD + FTS search)
│   │   ├── user.rs          # User model
│   │   ├── category.rs      # Category model
│   │   ├── tag.rs           # Tag model
│   │   ├── media.rs         # Media model
│   │   └── setting.rs       # Site settings KV store
│   └── routes/
│       ├── mod.rs           # Public pages (index / search / setup / health)
│       ├── admin.rs         # Admin panel (posts / categories / tags / media / settings)
│       ├── posts.rs         # Public post pages
│       ├── taxonomy.rs      # Category & tag archive pages
│       ├── auth.rs          # Auth API (/auth/register, /auth/login)
│       └── api.rs           # REST API (/api/posts)
├── templates/
│   ├── base.html            # Public page base layout
│   ├── index.html           # Homepage
│   ├── search.html          # Full-text search results
│   ├── setup.html           # First-run setup wizard
│   ├── posts/               # Public post templates
│   ├── taxonomy/            # Category & tag archive templates
│   └── admin/               # Admin panel templates
├── migrations/              # SQLx auto-migrations (embedded in binary)
├── static/
│   └── uploads/             # Media upload directory
├── tests/                   # Integration tests (auth / posts / admin / api)
├── ai_docs/
│   ├── implementation-tasks.md       # Phase 1 completed tasks
│   └── wordpress-parity-tasks.md     # WordPress parity roadmap
├── .claude/
│   └── commands/
│       ├── wp-add-task.md   # /wp-add-task skill
│       └── wp-implement.md  # /wp-implement skill
└── Dockerfile
```

## 🌐 Endpoints

### Public

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Homepage |
| GET | `/posts` | Post list (paginated) |
| GET | `/posts/{slug}` | Single post |
| GET | `/categories/{slug}` | Category archive |
| GET | `/tags/{slug}` | Tag archive |
| GET | `/search?q=...` | Full-text search |
| GET | `/setup` | First-run setup wizard |
| GET | `/health` | Health check (`{"status":"ok"}`) |

### Auth

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/register` | Register user → issue JWT |
| POST | `/auth/login` | Login → issue JWT |

### Admin (login required)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/admin` | Dashboard |
| GET/POST | `/admin/posts` | Post list & create |
| GET/POST | `/admin/posts/{id}/edit` | Edit post |
| GET/POST | `/admin/categories` | Category management |
| GET/POST | `/admin/tags` | Tag management |
| GET/POST | `/admin/media` | Media library |
| GET/POST | `/admin/settings` | Site settings |

### REST API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/posts` | Post list (JSON) |
| GET | `/api/posts/{id}` | Single post (JSON) |
| POST | `/api/posts` | Create post (auth required) |
| PUT | `/api/posts/{id}` | Update post (auth required) |
| DELETE | `/api/posts/{id}` | Delete post (auth required) |

## 🤖 Claude Code Skills

This project includes **Claude Code custom commands** for AI-assisted development.  
The following slash commands are available in the `claude` CLI.

### `/wp-add-task [task description]`

Adds a new task to `ai_docs/wordpress-parity-tasks.md`.

```
/wp-add-task Auto-generate thumbnails for featured images
/wp-add-task          ← show all pending tasks when no argument given
```

### `/wp-implement [task name or phase]`

Implements a task from the list — runs `cargo build` and updates the checkbox automatically.

```
/wp-implement RSS feed
/wp-implement Phase 6      ← specify by phase number
/wp-implement              ← show all pending tasks when no argument given
```

## 📊 Implementation Status

| Feature | Status |
|---------|--------|
| Post CRUD + Markdown rendering | ✅ |
| Category & tag management | ✅ |
| Media upload & library | ✅ |
| JWT authentication | ✅ |
| REST API | ✅ |
| FTS5 full-text search | ✅ |
| Site settings | ✅ |
| Docker / health check | ✅ |
| Web installer (setup wizard) | ✅ |
| Static pages (Pages) | 🔲 |
| Featured images | 🔲 |
| Comment system | 🔲 |
| Custom navigation menus | 🔲 |
| RSS feed / XML sitemap | 🔲 |
| User roles & capabilities | 🔲 |
| Rich text editor | 🔲 |

For the full roadmap see [`ai_docs/wordpress-parity-tasks.md`](ai_docs/wordpress-parity-tasks.md).

## 🧪 Testing

```bash
cargo test
```

Integration tests: `tests/auth_test.rs` / `tests/admin_posts_test.rs` / `tests/public_posts_test.rs` / `tests/api_test.rs`

## 🪄 Philosophy

> "RedLeaf — grows naturally, powered by Rust."

Every page is a leaf.  
Every site is a tree.  
And Rust is the root that keeps it strong.

## 📜 License

MIT