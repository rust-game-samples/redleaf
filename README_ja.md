# 🌿 RedLeaf CMS

Rust で構築された軽量・高速な CMS — WordPress に近い機能をミニマルに実現。

> 🇺🇸 English documentation: [README.md](README.md)

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
│   ├── main.rs              # エントリポイント
│   ├── lib.rs               # ルーター構築
│   ├── auth.rs              # JWT 生成・検証
│   ├── db.rs                # SQLite 接続プール
│   ├── errors.rs            # 統一エラー型
│   ├── middleware.rs        # 認証ミドルウェア
│   ├── util.rs              # slugify / render / FTS ユーティリティ
│   ├── models/
│   │   ├── post.rs          # 投稿モデル (CRUD + FTS 検索)
│   │   ├── user.rs          # ユーザーモデル
│   │   ├── category.rs      # カテゴリモデル
│   │   ├── tag.rs           # タグモデル
│   │   ├── media.rs         # メディアモデル
│   │   └── setting.rs       # サイト設定 KV ストア
│   └── routes/
│       ├── mod.rs           # 公開ページ (index / search / setup / health)
│       ├── admin.rs         # 管理画面 (投稿・カテゴリ・タグ・メディア・設定)
│       ├── posts.rs         # 公開投稿ページ
│       ├── taxonomy.rs      # カテゴリ別・タグ別一覧
│       ├── auth.rs          # 認証 API (/auth/register, /auth/login)
│       └── api.rs           # REST API (/api/posts)
├── templates/
│   ├── base.html            # 公開ページ共通レイアウト
│   ├── index.html           # ホームページ
│   ├── search.html          # 全文検索結果
│   ├── setup.html           # 初回セットアップウィザード
│   ├── posts/               # 公開投稿テンプレート
│   ├── taxonomy/            # カテゴリ・タグ一覧
│   └── admin/               # 管理画面テンプレート
├── migrations/              # SQLx 自動マイグレーション（バイナリに埋め込み）
├── static/
│   └── uploads/             # メディアアップロード先
├── tests/                   # 統合テスト (auth / posts / admin / api)
├── ai_docs/
│   ├── implementation-tasks.md       # フェーズ 1 完了済みタスク
│   └── wordpress-parity-tasks.md     # WordPress 比較タスクリスト
├── .claude/
│   └── commands/
│       ├── wp-add-task.md   # /wp-add-task スキル
│       └── wp-implement.md  # /wp-implement スキル
└── Dockerfile
```

## 🌐 Endpoints

### Public

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | ホームページ |
| GET | `/posts` | 投稿一覧（ページネーション付き） |
| GET | `/posts/{slug}` | 個別投稿 |
| GET | `/categories/{slug}` | カテゴリ別一覧 |
| GET | `/tags/{slug}` | タグ別一覧 |
| GET | `/search?q=...` | 全文検索 |
| GET | `/setup` | 初回セットアップウィザード |
| GET | `/health` | ヘルスチェック (`{"status":"ok"}`) |

### Auth

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/register` | ユーザー登録 → JWT 発行 |
| POST | `/auth/login` | ログイン → JWT 発行 |

### Admin (要ログイン)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/admin` | ダッシュボード |
| GET/POST | `/admin/posts` | 投稿一覧・作成 |
| GET/POST | `/admin/posts/{id}/edit` | 投稿編集 |
| GET/POST | `/admin/categories` | カテゴリ管理 |
| GET/POST | `/admin/tags` | タグ管理 |
| GET/POST | `/admin/media` | メディアライブラリ |
| GET/POST | `/admin/settings` | サイト設定 |

### REST API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/posts` | 投稿一覧 JSON |
| GET | `/api/posts/{id}` | 個別投稿 JSON |
| POST | `/api/posts` | 投稿作成（認証必須） |
| PUT | `/api/posts/{id}` | 投稿更新（認証必須） |
| DELETE | `/api/posts/{id}` | 投稿削除（認証必須） |

## 🤖 Claude Code スキル

このプロジェクトには **Claude Code カスタムコマンド** が含まれています。  
`claude` CLI から以下のスラッシュコマンドが使用できます。

### `/wp-add-task [タスクの説明]`

WordPress parity タスクを `ai_docs/wordpress-parity-tasks.md` に追加します。

```
/wp-add-task アイキャッチ画像のサムネイル自動生成
/wp-add-task                     ← 引数なしで未完了タスク一覧を表示
```

### `/wp-implement [タスク名またはフェーズ]`

タスクリストから選んで実装します。ビルド確認とチェックマーク更新まで自動で行います。

```
/wp-implement RSS フィード
/wp-implement フェーズ 6        ← フェーズ単位で指定も可
/wp-implement                   ← 引数なしで未完了タスク一覧を表示
```

## 📊 実装状況

| カテゴリ | 状況 |
|---------|------|
| 投稿 CRUD + Markdown | ✅ |
| カテゴリ・タグ管理 | ✅ |
| メディアアップロード | ✅ |
| JWT 認証 | ✅ |
| REST API | ✅ |
| FTS5 全文検索 | ✅ |
| サイト設定 | ✅ |
| Docker / ヘルスチェック | ✅ |
| Webインストーラー | ✅ |
| 固定ページ (Pages) | 🔲 |
| アイキャッチ画像 | 🔲 |
| コメントシステム | 🔲 |
| カスタムメニュー | 🔲 |
| RSS フィード / サイトマップ | 🔲 |
| ユーザーロール・権限 | 🔲 |
| リッチテキストエディタ | 🔲 |

詳細は [`ai_docs/wordpress-parity-tasks.md`](ai_docs/wordpress-parity-tasks.md) を参照。

## 🧪 Testing

```bash
cargo test
```

統合テスト: `tests/auth_test.rs` / `tests/admin_posts_test.rs` / `tests/public_posts_test.rs` / `tests/api_test.rs`

## 🪄 Philosophy

> "RedLeaf — grows naturally, powered by Rust."

Every page is a leaf.  
Every site is a tree.  
And Rust is the root that keeps it strong.

## 📜 License

MIT