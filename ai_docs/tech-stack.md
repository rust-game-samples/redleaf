# RedLeaf CMS — 技術スタック

最終更新: 2026-05-29

---

## 言語・ランタイム

| 項目 | 内容 |
|---|---|
| 言語 | Rust (stable, 2021 edition) |
| 非同期ランタイム | Tokio 1.x (`full` feature) |

---

## Web フレームワーク

### Axum 0.8.6

- `Router` によるルートネスト・スコープ管理
- `State<DbPool>` で DB プールをハンドラに注入
- `Extension<T>` でページキャッシュ等を横断的に共有
- `Path<T>` / `Query<T>` / `Form<T>` / `Multipart` エクストラクタ
- `axum::middleware::from_fn` で Tower 互換ミドルウェアを実装

### Tower / tower-http 0.5.2 / 0.6.6

- `ServeDir` で `./static` を静的ファイル配信
- `TraceLayer` で HTTP リクエストの構造化ログ

---

## データベース

### SQLite (via SQLx 0.8.6)

| Feature | 内容 |
|---|---|
| `sqlite` | SQLite バックエンド |
| `runtime-tokio-native-tls` | 非同期 Tokio ランタイム |
| `migrate` | `sqlx::migrate!` でバイナリ埋め込みマイグレーション |
| `chrono` | `DateTime<Utc>` の自動マッピング |

- マイグレーション 15 件 (バイナリに埋め込み、起動時に自動適用)
- コネクションプール最大 5 接続
- FTS5 仮想テーブルで全文検索

> **注意**: 現状は SQLite 単体。本番では PostgreSQL 等への移行を想定。

---

## テンプレートエンジン

### Askama 0.14.0 + askama_axum 0.4

Jinja2 ライクな静的テンプレートエンジン (コンパイル時解決・ゼロランタイムコスト)。

- `#[derive(Template)]` + `#[template(path = "...")]` でハンドラから直接レンダリング
- `{% if let %}` / `{% for %}` / `{% block %}` / `{% include %}` / `{% extends %}` 対応
- カスタムフィルター: `rl_date` / `strip_markdown` 等を `src/filters.rs` で定義
- テーマ: `templates/themes/default/` に WordPress ライクなテンプレート階層

---

## 認証・セキュリティ

### Argon2 0.5.3

- アルゴリズム: Argon2id (メモリハード)
- 出力: PHC 文字列フォーマット
- 依存: `password-hash 0.5.0` / `rand_core 0.9.3` / `rand 0.9` / `getrandom 0.3.4`

### jsonwebtoken 10.1.0

- 署名アルゴリズム: HS256
- シークレット: 環境変数 `JWT_SECRET`
- Cookie `session` にトークンを保存 (HttpOnly / SameSite=Strict / Max-Age=7日)
- Claims: `sub` (user_id) / `role` / `exp`

### 権限システム

- 5ロール: administrator / editor / author / contributor / subscriber
- `has_capability(role, cap)` で細粒度チェック
- `require_auth` ミドルウェアで管理画面全体を保護

---

## コンテンツ処理

### pulldown-cmark 0.13.0

- CommonMark 仕様 Markdown パーサー
- 有効オプション: `ENABLE_STRIKETHROUGH` / `ENABLE_TABLES`
- ショートコード展開 → Markdown → HTML の変換パイプライン

### shortcodes (src/shortcodes.rs)

- `ShortcodeRegistry` で `[gallery]` / `[caption]` / `[audio]` 等を登録・展開

---

## 画像処理

### image 0.25 (features: webp)

- `image::load_from_memory` でアップロード画像を解析
- `resize_to_fill` (thumbnail 150px 正方形) / `resize` (medium 300px, large 1024px)
- `image-webp 0.2.4` (純 Rust) で WebP エンコーディング
- `tokio::task::spawn_blocking` でブロッキング処理を非同期ランタイムから分離
- バリアントは `media_variants` テーブルに保存
- 公開投稿ページでは `<picture>` + `<img srcset>` を自動出力

---

## XML 処理

### quick-xml 0.37

- WordPress WXR 1.2 エクスポートファイルのパース (`src/wxr.rs`)
- イベントベースパーサー (SAX ライク) で名前空間付き要素を処理
- `wp:` / `dc:` / `content:` / `excerpt:` 名前空間対応

---

## キャッシュ

### インメモリページキャッシュ (src/cache.rs)

- Tower 互換ミドルウェアとして実装 (`axum::middleware::from_fn`)
- 対象: 公開 GET リクエスト (HTML/XML/RSS レスポンス)
- 除外: `/admin` / `/api` / `/auth` / `/setup` / `/static`
- TTL: 5 分 / 最大ボディサイズ: 4 MB
- ETag (`DefaultHasher` ハッシュ) / Last-Modified ヘッダー出力
- 条件付き GET (`If-None-Match`) で 304 Not Modified を返す
- 投稿更新・削除・公開切り替え時に `invalidate_all()` を自動呼び出し

---

## シリアライゼーション

### Serde 1.x + serde_json 1.x

- `Serialize` / `Deserialize` derive で JSON・フォームデータを変換
- `#[serde(skip_serializing)]` でパスワードハッシュを除外
- `empty_string_as_none` / `one_or_many_i64` カスタムデシリアライザで HTML フォームの癖を吸収

### Chrono 0.4

- 全タイムスタンプを `DateTime<Utc>` で管理
- SQLx chrono feature で DB 日時フィールドと自動マッピング

---

## フックシステム

### hooks.rs

- `ActionRegistry` — `do_action(hook)` / `add_action(hook, callback, priority)`
- `FilterRegistry` — `apply_filters(hook, value)` / `add_filter(hook, callback, priority)`
- ビルトインフック: `before_post_save` / `after_post_publish` / `on_user_login`

---

## ロギング・観測性

### tracing 0.1 + tracing-subscriber 0.3

- `RUST_LOG` 環境変数でフィルタリング
- `TraceLayer` で HTTP アクセスログを自動記録
- 管理画面の操作は `activity_logs` テーブルにも記録

---

## エラーハンドリング

### AppError (src/errors.rs, thiserror 2.0.17)

```rust
pub enum AppError {
    NotFound,
    Database(sqlx::Error),
    Internal(String),
}
```

- `IntoResponse` 実装で HTTP ステータスに自動変換
- モデル層: `anyhow::Result<T>` で柔軟なエラー伝播

---

## ビルド設定

### build.rs

- `rustc --version` を実行して `RUST_VERSION` 環境変数に埋め込み
- ダッシュボードの「サイトヘルス」セクションで表示

### Cargo.toml release プロファイル

```toml
[profile.release]
opt-level     = 3    # 最大最適化
lto           = true # リンク時最適化
codegen-units = 1    # 単一コード生成ユニット (LTO 効果最大化)
```

---

## 依存クレート一覧

| クレート | バージョン | 用途 |
|---|---|---|
| axum | 0.8.6 | Web フレームワーク |
| tokio | 1.x | 非同期ランタイム |
| tower | 0.5.2 | ミドルウェア基盤 |
| tower-http | 0.6.6 | ServeDir / TraceLayer |
| sqlx | 0.8.6 | DB クライアント (SQLite) |
| askama | 0.14.0 | テンプレートエンジン |
| askama_axum | 0.4 | Askama × Axum 統合 |
| argon2 | 0.5.3 | パスワードハッシュ |
| password-hash | 0.5.0 | ハッシュトレイト |
| jsonwebtoken | 10.1.0 | JWT |
| rand_core | 0.9.3 | OS RNG |
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
| image | 0.25 | 画像リサイズ + WebP |
| bytes | 1.x | バイト列操作 |
| quick-xml | 0.37 | XML パース (WXR) |