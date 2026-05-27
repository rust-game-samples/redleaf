# RedLeaf CMS - データ構造

## データベーススキーマ

### users テーブル

```sql
CREATE TABLE users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT    NOT NULL UNIQUE,
    email         TEXT    NOT NULL UNIQUE,
    password_hash TEXT    NOT NULL,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL
);

CREATE INDEX idx_users_email    ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

### posts テーブル

```sql
CREATE TABLE posts (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    slug       TEXT    NOT NULL UNIQUE,
    content    TEXT    NOT NULL,
    excerpt    TEXT,                        -- NULL 可（任意の要約）
    published  INTEGER NOT NULL DEFAULT 0,  -- 0: 下書き, 1: 公開
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL
);

CREATE INDEX idx_posts_published ON posts(published);
CREATE INDEX idx_posts_slug      ON posts(slug);
```

**注**: posts と users 間の外部キー制約は現状なし（投稿者 `author_id` は未実装）。

---

## Rust 構造体

### User 関連 (`src/models/user.rs`)

```rust
// 永続化モデル（DBからの読み取り用）
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id:            i64,
    pub username:      String,
    pub email:         String,
    #[serde(skip)]             // シリアライズ時に除外
    pub password_hash: String,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

// ユーザー登録 DTO
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email:    String,
    pub password: String,    // プレーンテキスト（登録後にハッシュ化）
}

// ログイン DTO
#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email:    String,
    pub password: String,
}
```

### Post 関連 (`src/models/post.rs`)

```rust
// 永続化モデル
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id:         i64,
    pub title:      String,
    pub slug:       String,
    pub content:    String,
    pub excerpt:    Option<String>,
    pub published:  bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 投稿作成 DTO
#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title:     String,
    pub slug:      String,
    pub content:   String,
    pub excerpt:   Option<String>,
    pub published: bool,
}

// 投稿更新 DTO（全フィールドが Optional = 部分更新）
#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title:     Option<String>,
    pub slug:      Option<String>,
    pub content:   Option<String>,
    pub excerpt:   Option<String>,
    pub published: Option<bool>,
}
```

---

## データ操作メソッド一覧

### User メソッド

| メソッド | シグネチャ | 説明 |
|---|---|---|
| `hash_password` | `(password: &str) -> Result<String>` | Argon2 ハッシュ生成 |
| `verify_password` | `(password: &str, hash: &str) -> Result<bool>` | パスワード検証 |
| `find_by_id` | `(pool, id: i64) -> Result<Option<User>>` | ID で検索 |
| `find_by_email` | `(pool, email: &str) -> Result<Option<User>>` | メールで検索 |
| `find_by_username` | `(pool, username: &str) -> Result<Option<User>>` | ユーザー名で検索 |
| `create` | `(pool, CreateUser) -> Result<User>` | ユーザー作成 |
| `authenticate` | `(pool, LoginUser) -> Result<Option<User>>` | ログイン認証 |

### Post メソッド

| メソッド | シグネチャ | 説明 |
|---|---|---|
| `find_all` | `(pool) -> Result<Vec<Post>>` | 公開済み投稿一覧（新着順） |
| `find_by_id` | `(pool, id: i64) -> Result<Option<Post>>` | ID で検索 |
| `find_by_slug` | `(pool, slug: &str) -> Result<Option<Post>>` | スラッグで検索 |
| `create` | `(pool, CreatePost) -> Result<Post>` | 投稿作成 |
| `update` | `(pool, id: i64, UpdatePost) -> Result<Post>` | 部分更新 |
| `delete` | `(pool, id: i64) -> Result<()>` | 投稿削除 |

---

## データ型マッピング

| SQLite 型 | Rust 型 | 備考 |
|---|---|---|
| `INTEGER PRIMARY KEY` | `i64` | 自動インクリメント |
| `TEXT NOT NULL` | `String` | 必須文字列 |
| `TEXT` (NULL 可) | `Option<String>` | 任意文字列 |
| `INTEGER NOT NULL DEFAULT 0` | `bool` | SQLite では 0/1 |
| `TEXT NOT NULL` (日時) | `DateTime<Utc>` | ISO 8601 文字列として保存 |

---

## 将来的な拡張フィールド（未実装）

### posts テーブルへの追加候補

```sql
author_id    INTEGER REFERENCES users(id),  -- 投稿者
category_id  INTEGER REFERENCES categories(id),
tags         TEXT,                           -- JSON 配列 or 別テーブル
cover_image  TEXT,                           -- 画像パス or URL
view_count   INTEGER DEFAULT 0,
```

### 新規テーブル候補

- `categories` - カテゴリ管理
- `tags` / `post_tags` - タグ管理（多対多）
- `media` - アップロードファイル管理
- `settings` - サイト設定 KV ストア
- `sessions` - JWT リフレッシュトークン管理