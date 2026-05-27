# RedLeaf CMS - 技術スタック

## 言語・ランタイム

| 項目 | 内容 |
|---|---|
| 言語 | Rust (2021 edition) |
| 非同期ランタイム | Tokio 1.x (`full` feature) |

---

## Web フレームワーク

### Axum 0.8.6

Tokio ベースの軽量非同期 HTTP フレームワーク。

- `Router` によるルートネスト
- `State<T>` エクストラクタで DB プールを注入
- `Path<T>` / `Json<T>` / `Form<T>` エクストラクタ
- `IntoResponse` トレイトでハンドラの戻り値を柔軟に定義

### Tower / tower-http 0.5.2 / 0.6.6

Axum のミドルウェア基盤。

- `ServeDir` で `./static` を静的ファイル配信
- `TraceLayer` で HTTP リクエストの構造化ログ

---

## データベース

### SQLite (via SQLx 0.8.6)

| Feature | 内容 |
|---|---|
| `sqlite` | SQLite バックエンド |
| `runtime-tokio-native-tls` | 非同期 Tokio ランタイム |
| `migrate` | `sqlx::migrate!` でスキーマ自動適用 |
| `chrono` | `DateTime<Utc>` の自動マッピング |

**SQLx の特徴**:
- コンパイル時 SQL チェック（型安全なクエリ）
- `FromRow` derive でクエリ結果を構造体に直接マッピング
- コネクションプール（最大 5 接続）

---

## テンプレートエンジン

### Askama 0.14.0 + askama_axum 0.4

Jinja2 ライクな静的テンプレートエンジン（コンパイル時解決）。

**現状**: 依存に含まれているが未使用。HTML はハンドラ内インライン文字列で生成。  
**予定**: `templates/` ディレクトリに `.html` ファイルを追加して移行。

---

## 認証・セキュリティ

### Argon2 0.5.3

メモリハードなパスワードハッシュアルゴリズム。

- アルゴリズム: Argon2id
- 出力: PHC 文字列フォーマット（`$argon2id$v=19$...`）
- 依存: `password-hash 0.5.0`（ハッシュトレイト）、`rand_core 0.9.3`・`rand 0.9`・`getrandom 0.3.4`（乱数生成）

### jsonwebtoken 10.1.0

JWT (JSON Web Token) ライブラリ。

- 署名アルゴリズム: HS256 を想定
- シークレット: 環境変数 `JWT_SECRET`
- **現状**: クレートは導入済みだがトークン生成・検証は未実装

---

## コンテンツ処理

### pulldown-cmark 0.13.0

CommonMark 仕様の Markdown パーサー。

- 有効オプション: `ENABLE_STRIKETHROUGH`・`ENABLE_TABLES`
- 投稿コンテンツを HTML へ変換して表示

---

## シリアライゼーション

### Serde 1.x + serde_json 1.x

- `Serialize` / `Deserialize` derive でリクエスト/レスポンスの JSON 変換
- `#[serde(skip)]` でパスワードハッシュをシリアライズから除外

### Chrono 0.4

- `DateTime<Utc>` で全タイムスタンプを UTC 管理
- SQLx chrono feature でDB日時フィールドと自動マッピング

---

## ロギング・観測性

### tracing 0.1 + tracing-subscriber 0.3

- 構造化ログ（スパン・イベントベース）
- `RUST_LOG` 環境変数でフィルタリング（例: `debug`, `info`）
- tower-http `TraceLayer` で HTTP アクセスログを自動記録

---

## エラーハンドリング

### anyhow 1.x

- `anyhow::Result<T>` でモデル層のエラーを柔軟に伝播
- ボックス化されたエラー型でライブラリ間の型変換不要

### thiserror 2.0.17

- `#[derive(Error)]` で独自エラー型を定義
- **現状**: 依存に含まれているが未使用（将来の統一エラー型向け）

---

## 設定管理

### dotenvy 0.15

- 起動時に `.env` ファイルを自動読み込み
- `std::env::var` でアクセス

---

## ビルド設定 (Cargo.toml)

```toml
[profile.release]
opt-level      = 3    # 最大最適化
lto            = true # リンク時最適化（バイナリサイズ削減・速度向上）
codegen-units  = 1    # 単一コード生成ユニット（LTO 効果最大化）
```

---

## スタック全体図

```
┌─────────────────────────────────────────────────────┐
│  クライアント（ブラウザ / API クライアント）           │
└────────────────┬────────────────────────────────────┘
                 │ HTTP
┌────────────────▼────────────────────────────────────┐
│  Axum 0.8.6 + Tower / tower-http                    │
│  ・ServeDir（静的ファイル）                           │
│  ・TraceLayer（アクセスログ）                         │
└────────────────┬────────────────────────────────────┘
                 │ Router / Handler
┌────────────────▼────────────────────────────────────┐
│  ハンドラ層 (routes/)                                │
│  ・Askama テンプレート（予定）or インライン HTML       │
│  ・pulldown-cmark（Markdown→HTML）                   │
└────────────────┬────────────────────────────────────┘
                 │ メソッド呼び出し
┌────────────────▼────────────────────────────────────┐
│  モデル層 (models/)                                  │
│  ・Argon2 パスワードハッシュ                          │
│  ・jsonwebtoken JWT（未実装）                         │
│  ・serde JSON シリアライズ                            │
└────────────────┬────────────────────────────────────┘
                 │ SQLx クエリ
┌────────────────▼────────────────────────────────────┐
│  SQLite (redleaf.db)                                │
│  ・SQLx 非同期プール（最大 5 接続）                   │
│  ・sqlx::migrate! でスキーマ管理                     │
└─────────────────────────────────────────────────────┘
```

---

## バージョン一覧

| クレート | バージョン | 用途 |
|---|---|---|
| axum | 0.8.6 | Web フレームワーク |
| tokio | 1.x | 非同期ランタイム |
| tower | 0.5.2 | ミドルウェア |
| tower-http | 0.6.6 | HTTP ミドルウェア |
| sqlx | 0.8.6 | DB クライアント（SQLite） |
| askama | 0.14.0 | テンプレートエンジン |
| askama_axum | 0.4 | Askama Axum 統合 |
| argon2 | 0.5.3 | パスワードハッシュ |
| password-hash | 0.5.0 | ハッシュトレイト |
| jsonwebtoken | 10.1.0 | JWT |
| rand_core | 0.9.3 | 乱数生成 |
| rand | 0.9 | 乱数生成 |
| getrandom | 0.3.4 | OS RNG |
| serde | 1.x | シリアライゼーション |
| serde_json | 1.x | JSON |
| chrono | 0.4 | 日時 |
| pulldown-cmark | 0.13.0 | Markdown パーサー |
| dotenvy | 0.15 | .env 読み込み |
| tracing | 0.1 | 構造化ログ |
| tracing-subscriber | 0.3 | ログ出力 |
| anyhow | 1.x | エラーハンドリング |
| thiserror | 2.0.17 | エラー型定義 |