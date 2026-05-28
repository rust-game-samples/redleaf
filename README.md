# 🌿 RedLeaf CMS

A lightweight, blazing-fast CMS built with Rust — WordPress parity without the bloat.

> 🇯🇵 Japanese documentation: [README_ja.md](README_ja.md)

## 🚀 About

**RedLeaf** is a modern CMS powered by Rust.  
It combines the **stability of systems programming** with the **flexibility of web publishing**.

- ⚡ **Fast** — compiled Rust backend, in-memory page cache, minimal runtime overhead
- 🪶 **Lightweight** — single binary + SQLite, zero external runtime dependencies
- 🔒 **Secure** — Argon2id password hashing, JWT auth, role-based capabilities
- 🌐 **Headless Ready** — full REST API included
- 🐳 **Docker Ready** — multi-stage build, production-grade image
- 🧩 **WordPress Compatible** — WXR import/export, familiar admin UX

## 🔧 Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (stable, 2021 edition) |
| Web Framework | [Axum](https://github.com/tokio-rs/axum) 0.8 |
| Database | SQLite via [SQLx](https://github.com/launchbadge/sqlx) 0.8 |
| Templates | [Askama](https://github.com/djc/askama) 0.14 (compiled at build time) |
| Auth | JWT ([jsonwebtoken](https://github.com/Keats/jsonwebtoken)) + Argon2id |
| Search | SQLite FTS5 (full-text search) |
| Image Processing | [image](https://github.com/image-rs/image) 0.25 (resize + WebP) |
| XML Parsing | [quick-xml](https://github.com/tafia/quick-xml) 0.37 (WXR import) |
| Cache | In-memory page cache (Tower middleware) |

## ⚙️ Quick Start

### Development

```bash
git clone https://github.com/yourname/redleaf.git
cd redleaf
cp .env.example .env   # set JWT_SECRET before production use
cargo run
```

Open http://localhost:3000  
First run: visit http://localhost:3000/setup to create the admin account.

### Docker

```bash
docker build -t redleaf .
docker run -p 3000:3000 \
  -v redleaf-data:/app/data \
  -v redleaf-uploads:/app/static/uploads \
  -e JWT_SECRET=$(openssl rand -hex 32) \
  redleaf
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:redleaf.db` | SQLite file path |
| `HOST` | `127.0.0.1` | Bind address |
| `PORT` | `3000` | Listen port |
| `JWT_SECRET` | *(required in production)* | Token signing secret |

## 🗂️ Directory Structure

```
redleaf/
├── src/
│   ├── main.rs              # Entry point — server startup
│   ├── lib.rs               # App builder — router + middleware wiring
│   ├── auth.rs              # JWT generation & validation
│   ├── cache.rs             # In-memory page cache (Tower middleware)
│   ├── db.rs                # SQLite connection pool
│   ├── errors.rs            # Unified AppError type
│   ├── filters.rs           # Askama template filters
│   ├── hooks.rs             # Action/filter hook registry (WordPress-style)
│   ├── image_processing.rs  # Image resize + WebP variant generation
│   ├── middleware.rs        # Auth middleware + capability checks
│   ├── shortcodes.rs        # Shortcode registry ([gallery], [caption], …)
│   ├── util.rs              # slugify / render / Pagination / FTS helpers
│   ├── wxr.rs               # WordPress WXR XML parser
│   ├── assets.rs            # Script/style enqueue registry
│   ├── models/
│   │   ├── activity_log.rs  # Admin activity log
│   │   ├── category.rs      # Categories
│   │   ├── comment.rs       # Comments (threaded)
│   │   ├── media.rs         # Media library + image variants
│   │   ├── nav_menu.rs      # Custom navigation menus
│   │   ├── page.rs          # Static pages
│   │   ├── post.rs          # Posts (CRUD, FTS, revisions, scheduling)
│   │   ├── post_meta.rs     # Custom fields (KV store per post)
│   │   ├── post_revision.rs # Post revision history
│   │   ├── setting.rs       # Site settings KV store
│   │   ├── tag.rs           # Tags
│   │   ├── user.rs          # Users (roles, profiles)
│   │   └── widget.rs        # Widget areas + widgets
│   └── routes/
│       ├── mod.rs           # Public pages (home, search, setup, sitemap, …)
│       ├── admin.rs         # Admin panel (all CRUD + import/export)
│       ├── api.rs           # REST API (/api/posts)
│       ├── auth.rs          # Auth API (/auth/login, /auth/register)
│       ├── feed.rs          # RSS 2.0 & Atom feeds
│       ├── posts.rs         # Public post pages
│       └── taxonomy.rs      # Category & tag archive pages
├── templates/
│   ├── admin/               # Admin panel (dashboard, posts, media, …)
│   ├── themes/default/      # Default public theme (single, archive, …)
│   ├── base.html            # Public base layout
│   ├── index.html           # Homepage
│   ├── search.html          # Search results
│   └── setup.html           # First-run setup wizard
├── migrations/              # 15 SQLx migrations (embedded in binary)
├── static/
│   └── uploads/             # User-uploaded media files
├── tests/                   # Integration tests
├── ai_docs/                 # Project documentation & Claude Code skills
├── build.rs                 # Captures RUST_VERSION at compile time
└── Dockerfile
```

## 🌐 Endpoints

### Public

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Homepage |
| GET | `/posts` | Post archive (paginated) |
| GET | `/posts/{slug}` | Single post |
| GET | `/categories/{slug}` | Category archive |
| GET | `/tags/{slug}` | Tag archive |
| GET | `/author/{username}` | Author archive |
| GET | `/pages/{slug}` | Static page |
| GET | `/search?q=…` | Full-text search |
| GET | `/feed` | RSS 2.0 feed |
| GET | `/feed/atom` | Atom feed |
| GET | `/sitemap.xml` | XML sitemap |
| GET | `/robots.txt` | robots.txt (editable in admin) |
| GET | `/health` | Health check (`{"status":"ok"}`) |

### Auth

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/login` | Login → JWT |
| POST | `/auth/register` | Register → JWT |

### Admin (session required)

| Area | Paths |
|------|-------|
| Dashboard | `GET /admin` (+ quick draft `POST`) |
| Posts | `/admin/posts` CRUD + bulk actions, toggle, revisions |
| Pages | `/admin/pages` CRUD |
| Categories | `/admin/categories` CRUD + bulk delete |
| Tags | `/admin/tags` + bulk delete |
| Media | `/admin/media` upload/delete (auto-generates variants) |
| Comments | `/admin/comments` approve/reject/spam |
| Users | `/admin/users` + role management |
| Menus | `/admin/menus` CRUD + drag-and-drop items |
| Widgets | `/admin/widgets` CRUD + reorder |
| Settings | `/admin/settings` + robots.txt |
| Activity Log | `GET /admin/activity-logs` |
| Import | `GET/POST /admin/import` (WXR) |
| Export | `GET /admin/export` → JSON / WXR / SQLite backup |

### REST API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/posts` | Post list (JSON) |
| GET | `/api/posts/{id}` | Single post (JSON) |
| POST | `/api/posts` | Create post *(auth)* |
| PUT | `/api/posts/{id}` | Update post *(auth)* |
| DELETE | `/api/posts/{id}` | Delete post *(auth)* |
| GET | `/api/users/{id}/posts` | Posts by user |

## 📊 Implementation Status

| Feature | Status |
|---------|--------|
| Post CRUD + Markdown rendering | ✅ |
| Category & tag management | ✅ |
| Media upload & library | ✅ |
| Image resize + WebP variants + `<img srcset>` | ✅ |
| JWT authentication + role-based capabilities | ✅ |
| REST API | ✅ |
| FTS5 full-text search | ✅ |
| Site settings | ✅ |
| Docker / health check | ✅ |
| Web installer (setup wizard) | ✅ |
| Static pages (Pages) | ✅ |
| Featured images + OGP | ✅ |
| Custom fields (Post Meta) | ✅ |
| Scheduled posts | ✅ |
| Post revisions | ✅ |
| Sticky posts | ✅ |
| Template system (themes) | ✅ |
| Hook system (actions / filters) | ✅ |
| Shortcode API (`[gallery]`, `[caption]`, `[audio]`) | ✅ |
| Custom navigation menus | ✅ |
| Breadcrumbs + JSON-LD | ✅ |
| Widget areas | ✅ |
| User roles & profiles | ✅ |
| Author archive pages | ✅ |
| Comment system (threaded) | ✅ |
| Comment moderation | ✅ |
| RSS 2.0 / Atom feeds | ✅ |
| XML sitemap | ✅ |
| SEO meta + Open Graph + Twitter Card | ✅ |
| Structured data (JSON-LD Article) | ✅ |
| Bulk actions (posts / categories / tags) | ✅ |
| Activity log | ✅ |
| Dashboard (stats / quick draft / site health) | ✅ |
| In-memory page cache + ETag / Last-Modified | ✅ |
| WordPress WXR import (+ dedup / slug rename) | ✅ |
| JSON export / WXR export / SQLite backup | ✅ |

## 🤖 Claude Code Skills

Custom commands for AI-assisted development:

### `/wp-implement [task or phase]`

Implements a task from `ai_docs/wordpress-parity-tasks.md`, runs `cargo build`, and marks the checkbox.

```
/wp-implement フェーズ 8
/wp-implement RSS feed
/wp-implement          ← list all pending tasks
```

### `/wp-add-task [description]`

Appends a new task to the roadmap file.

## 🧪 Testing

```bash
cargo test
```

Integration tests in `tests/`: `auth_test.rs` · `admin_posts_test.rs` · `public_posts_test.rs` · `api_test.rs` · `taxonomy_test.rs`

## 🚢 Deployment

### VPS (systemd)

```bash
cargo build --release
# copy binary + static/ to /opt/redleaf
# configure /etc/systemd/system/redleaf.service with JWT_SECRET env var
# put Nginx in front for TLS + static file serving
```

### Fly.io (recommended for SQLite)

```bash
fly launch --no-deploy
fly volumes create redleaf_data --size 1
fly volumes create redleaf_uploads --size 5
fly secrets set JWT_SECRET="$(openssl rand -hex 32)"
fly deploy
# visit https://your-app.fly.dev/setup
```

### Backup

Download a live SQLite snapshot anytime from **Admin → Export → Download DB**.

## 🪄 Philosophy

> "RedLeaf — grows naturally, powered by Rust."

Every page is a leaf. Every site is a tree. And Rust is the root that keeps it strong.

## 📜 License

MIT