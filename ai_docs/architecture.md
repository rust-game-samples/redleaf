# RedLeaf CMS - アーキテクチャ

## 概要

RedLeaf は Rust 製の軽量 CMS。WordPress ライクなブログ管理システムを目指し、パフォーマンス・シンプルさ・API ファーストを設計方針とする。

## レイヤー構成

```
┌─────────────────────────────────────────┐
│             Web Layer (routes/)          │  ← HTTP リクエスト処理・HTML 生成
├─────────────────────────────────────────┤
│            Model Layer (models/)         │  ← ビジネスロジック・データ構造
├─────────────────────────────────────────┤
│           Data Access Layer (db.rs)      │  ← SQLite 接続プール管理
├─────────────────────────────────────────┤
│            SQLite Database               │  ← 永続化
└─────────────────────────────────────────┘
```

## 各レイヤーの責務

### Web Layer (`src/routes/`)

| ファイル | 担当 |
|---|---|
| `routes/mod.rs` | ホームページハンドラ、ルートモジュール集約 |
| `routes/posts.rs` | 投稿一覧・個別表示、Markdown→HTML 変換 |
| `routes/admin.rs` | 管理画面ダッシュボード（骨格のみ） |

- Axum の `Router` を構築してネストされたルートグループを組む
- `State<DbPool>` でコネクションプールをハンドラに注入
- HTML はインライン文字列で生成（Askama は依存に含むが未使用）

### Model Layer (`src/models/`)

| ファイル | 担当 |
|---|---|
| `models/user.rs` | ユーザー認証・CRUD（Argon2 ハッシュ） |
| `models/post.rs` | ブログ投稿の CRUD・公開制御 |
| `models/mod.rs` | `User`・`Post` の再エクスポート |

- 各モデルは自身のデータベース操作メソッドを持つ（Active Record パターンに近い）
- SQLx の `FromRow` で SQL 結果を直接構造体へマッピング

### Data Access Layer (`src/db.rs`)

- `SqlitePool` の初期化のみを担当
- 最大 5 接続のプールを生成し `main.rs` へ返す

## リクエストフロー

### ブログ投稿表示

```
GET /posts/{id}
  └→ show_post(Path<id>, State<DbPool>)
       ├→ Post::find_by_id(&pool, id)
       │    └→ SELECT * FROM posts WHERE id = ?
       ├→ markdown_to_html(&post.content)   // pulldown-cmark
       └→ Html レスポンス返却
```

### ユーザー登録（実装中）

```
POST /register
  └→ CreateUser デシリアライズ
       ├→ User::hash_password(&password)   // Argon2 + OS RNG
       ├→ User::create(&pool, create_user)
       │    └→ INSERT INTO users (...)
       └→ User 返却（password_hash は serde skip）
```

## ルーティング構造

```
/                  → index()          ホームページ
/posts/            → list_posts()     投稿一覧
/posts/{id}        → show_post()      個別投稿
/admin/            → admin_dashboard() 管理画面
/static/*          → ServeDir         静的ファイル配信
```

## 状態管理

Axum の `Extension` / `State` を使い `DbPool` をグローバル共有。ハンドラ引数に `State<DbPool>` を宣言するだけで注入される。

## エラーハンドリング方針

- モデル層：`anyhow::Result<T>` で柔軟なエラー型を使用
- ハンドラ層：エラーを `tracing` でログ出力し、HTTP 500 または空レスポンスを返す
- 今後の課題：統一エラー型（`thiserror` ベース）への移行

## セキュリティ設計

| 項目 | 実装 |
|---|---|
| パスワード保存 | Argon2id（メモリハード・PHC 文字列形式） |
| SQL インジェクション | SQLx プリペアドステートメント |
| JWT 認証 | `jsonwebtoken` クレート（トークン生成は未実装） |
| 管理画面保護 | 認証ミドルウェア（未実装） |