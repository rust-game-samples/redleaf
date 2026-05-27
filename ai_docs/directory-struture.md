# RedLeaf CMS - ディレクトリ構造

## 現状のツリー

```
redleaf/
├── src/
│   ├── main.rs                         # エントリーポイント・サーバー起動・ルーティング設定
│   ├── db.rs                           # SQLite コネクションプール初期化
│   ├── models/
│   │   ├── mod.rs                      # User・Post の再エクスポート
│   │   ├── user.rs                     # ユーザー構造体・認証ロジック・CRUD
│   │   └── post.rs                     # 投稿構造体・CRUD・公開制御
│   └── routes/
│       ├── mod.rs                      # ホームページハンドラ・ルートモジュール集約
│       ├── posts.rs                    # 投稿一覧・個別表示・Markdown 変換
│       └── admin.rs                    # 管理ダッシュボード（骨格）
│
├── migrations/
│   └── 20240101000000_initial.sql     # 初期スキーマ（users・posts テーブル）
│
├── static/                             # 静的ファイル配信ディレクトリ（現在空）
│
├── ai_docs/                            # AI 向けドキュメント（本ディレクトリ）
│   ├── architecture.md
│   ├── data-struture.md
│   ├── directory-struture.md
│   ├── implementation-tasks.md
│   ├── requirement.md
│   └── tech-stack.md
│
├── .env                                # 環境変数（Git 管理 ※本番では除外すること）
├── .env.example                        # 環境変数テンプレート
├── .gitignore                          # Git 除外設定
├── Cargo.toml                          # パッケージ定義・依存関係
├── Cargo.lock                          # 依存バージョンロックファイル
├── redleaf.db                          # SQLite データベースファイル
└── README.md                           # プロジェクト説明
```

---

## 各ファイル詳細

### `src/main.rs` (約 53 行)

- `dotenvy` で `.env` を読み込み
- `tracing_subscriber` でログ初期化
- `db::init_db()` でプール取得
- `sqlx::migrate!()` でマイグレーション実行
- Axum `Router` 構築・`/posts`・`/admin`・`/static` をネスト
- `TcpListener` で `HOST:PORT` にバインドして `axum::serve` 起動

### `src/db.rs` (約 18 行)

- `pub type DbPool = SqlitePool`
- `pub async fn init_db() -> Result<DbPool>` のみ定義

### `src/models/user.rs` (約 130 行)

- `User` / `CreateUser` / `LoginUser` 構造体
- Argon2 ハッシュ・検証ヘルパー
- SQLx クエリメソッド（find 系・create・authenticate）

### `src/models/post.rs` (約 136 行)

- `Post` / `CreatePost` / `UpdatePost` 構造体
- 投稿の CRUD 全メソッド
- `find_all` は `published = 1` のみ返す

### `src/routes/mod.rs` (約 87 行)

- `pub fn router() -> Router` でホームルート定義
- `index()` ハンドラ：インライン HTML のランディングページ

### `src/routes/posts.rs` (約 177 行)

- `pub fn post_routes() -> Router`
- `list_posts()` / `show_post()` ハンドラ
- `markdown_to_html()` ヘルパー（pulldown-cmark 使用）

### `src/routes/admin.rs` (約 106 行)

- `pub fn admin_routes() -> Router`
- `admin_dashboard()` ハンドラ：ダッシュボードの骨格 HTML

### `migrations/20240101000000_initial.sql` (約 28 行)

- `users` テーブル + インデックス
- `posts` テーブル + インデックス

---

## 追加が想定されるパス（未実装）

```
src/
├── middleware/
│   ├── auth.rs          # JWT 認証ミドルウェア
│   └── mod.rs
├── api/
│   ├── mod.rs
│   ├── posts.rs         # REST API エンドポイント
│   └── users.rs
└── errors.rs            # 統一エラー型

templates/               # Askama テンプレートファイル（.html）
├── base.html
├── index.html
├── posts/
│   ├── list.html
│   └── show.html
└── admin/
    ├── dashboard.html
    └── posts/
        ├── index.html
        ├── new.html
        └── edit.html

static/
├── css/
│   └── style.css
├── js/
│   └── app.js
└── uploads/             # アップロードファイル

migrations/
├── 20240101000000_initial.sql
└── 20240201000000_add_author.sql   # 投稿者フィールド追加予定
```

---

## ソースコード規模

| ファイル | 行数 |
|---|---|
| src/main.rs | ~53 |
| src/db.rs | ~18 |
| src/models/user.rs | ~130 |
| src/models/post.rs | ~136 |
| src/models/mod.rs | ~6 |
| src/routes/mod.rs | ~87 |
| src/routes/posts.rs | ~177 |
| src/routes/admin.rs | ~106 |
| **合計** | **~713** |