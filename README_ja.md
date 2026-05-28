# 🌿 RedLeaf CMS

Rust で構築された軽量・高速な CMS — WordPress に近い機能をミニマルに実現。

> 🇺🇸 English documentation: [README.md](README.md)

## 🚀 概要

**RedLeaf** は Rust 製のモダン CMS です。  
システムプログラミングの堅牢性と Web パブリッシングの柔軟性を組み合わせています。

- ⚡ **高速** — コンパイル済み Rust バックエンド、インメモリページキャッシュ
- 🪶 **軽量** — バイナリ1本 + SQLite、外部ランタイム依存ゼロ
- 🔒 **安全** — Argon2id パスワードハッシュ、JWT 認証、ロールベース権限
- 🌐 **ヘッドレス対応** — REST API 完備
- 🐳 **Docker 対応** — マルチステージビルド、本番グレードイメージ
- 🧩 **WordPress 互換** — WXR インポート・エクスポート、使い慣れた管理 UX

## 🔧 技術スタック

| レイヤー | 技術 |
|-------|-----------|
| 言語 | Rust (stable, 2021 edition) |
| Web フレームワーク | [Axum](https://github.com/tokio-rs/axum) 0.8 |
| データベース | SQLite via [SQLx](https://github.com/launchbadge/sqlx) 0.8 |
| テンプレート | [Askama](https://github.com/djc/askama) 0.14 (ビルド時コンパイル) |
| 認証 | JWT ([jsonwebtoken](https://github.com/Keats/jsonwebtoken)) + Argon2id |
| 検索 | SQLite FTS5 全文検索 |
| 画像処理 | [image](https://github.com/image-rs/image) 0.25 (リサイズ + WebP) |
| XML パース | [quick-xml](https://github.com/tafia/quick-xml) 0.37 (WXR インポート) |
| キャッシュ | インメモリページキャッシュ (Tower ミドルウェア) |

## ⚙️ クイックスタート

### 開発環境

```bash
git clone https://github.com/yourname/redleaf.git
cd redleaf
cp .env.example .env   # 本番前に JWT_SECRET を変更すること
cargo run
```

http://localhost:3000 を開く  
初回: http://localhost:3000/setup で管理者アカウントを作成。

### Docker

```bash
docker build -t redleaf .
docker run -p 3000:3000 \
  -v redleaf-data:/app/data \
  -v redleaf-uploads:/app/static/uploads \
  -e JWT_SECRET=$(openssl rand -hex 32) \
  redleaf
```

### 環境変数

| 変数 | デフォルト | 説明 |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:redleaf.db` | SQLite ファイルパス |
| `HOST` | `127.0.0.1` | バインドアドレス |
| `PORT` | `3000` | ポート番号 |
| `JWT_SECRET` | *(本番では必須)* | トークン署名シークレット |

## 🗂️ ディレクトリ構造

```
redleaf/
├── src/
│   ├── main.rs              # エントリポイント — サーバー起動
│   ├── lib.rs               # アプリビルダー — ルーター + ミドルウェア配線
│   ├── auth.rs              # JWT 生成・検証
│   ├── cache.rs             # インメモリページキャッシュ (Tower ミドルウェア)
│   ├── db.rs                # SQLite コネクションプール
│   ├── errors.rs            # 統一エラー型 AppError
│   ├── filters.rs           # Askama テンプレートフィルター
│   ├── hooks.rs             # アクション/フィルターフックレジストリ (WordPress 風)
│   ├── image_processing.rs  # 画像リサイズ + WebP バリアント生成
│   ├── middleware.rs        # 認証ミドルウェア + 権限チェック
│   ├── shortcodes.rs        # ショートコードレジストリ ([gallery] 等)
│   ├── util.rs              # slugify / render / Pagination / FTS ヘルパー
│   ├── wxr.rs               # WordPress WXR XML パーサー
│   ├── assets.rs            # スクリプト/スタイルエンキューレジストリ
│   ├── models/
│   │   ├── activity_log.rs  # 管理アクティビティログ
│   │   ├── category.rs      # カテゴリ
│   │   ├── comment.rs       # コメント (スレッド対応)
│   │   ├── media.rs         # メディアライブラリ + 画像バリアント
│   │   ├── nav_menu.rs      # カスタムナビゲーションメニュー
│   │   ├── page.rs          # 固定ページ
│   │   ├── post.rs          # 投稿 (CRUD・FTS・リビジョン・スケジュール)
│   │   ├── post_meta.rs     # カスタムフィールド (KV ストア)
│   │   ├── post_revision.rs # 投稿リビジョン履歴
│   │   ├── setting.rs       # サイト設定 KV ストア
│   │   ├── tag.rs           # タグ
│   │   ├── user.rs          # ユーザー (ロール・プロフィール)
│   │   └── widget.rs        # ウィジェットエリア + ウィジェット
│   └── routes/
│       ├── mod.rs           # 公開ページ (ホーム・検索・セットアップ・サイトマップ等)
│       ├── admin.rs         # 管理画面 (全 CRUD + インポート/エクスポート)
│       ├── api.rs           # REST API (/api/posts)
│       ├── auth.rs          # 認証 API (/auth/login, /auth/register)
│       ├── feed.rs          # RSS 2.0 / Atom フィード
│       ├── posts.rs         # 公開投稿ページ
│       └── taxonomy.rs      # カテゴリ・タグアーカイブ
├── templates/
│   ├── admin/               # 管理画面テンプレート群
│   ├── themes/default/      # デフォルト公開テーマ (single, archive 等)
│   ├── base.html            # 公開ページ共通レイアウト
│   ├── index.html           # ホームページ
│   ├── search.html          # 検索結果
│   └── setup.html           # 初回セットアップウィザード
├── migrations/              # SQLx マイグレーション 15 件 (バイナリに埋め込み)
├── static/
│   └── uploads/             # アップロードされたメディアファイル
├── tests/                   # 統合テスト
├── ai_docs/                 # プロジェクトドキュメント & Claude Code スキル
├── build.rs                 # ビルド時に RUST_VERSION を埋め込む
└── Dockerfile
```

## 🌐 エンドポイント

### 公開

| メソッド | パス | 説明 |
|--------|------|-------------|
| GET | `/` | ホームページ |
| GET | `/posts` | 投稿アーカイブ (ページネーション付き) |
| GET | `/posts/{slug}` | 個別投稿 |
| GET | `/categories/{slug}` | カテゴリアーカイブ |
| GET | `/tags/{slug}` | タグアーカイブ |
| GET | `/author/{username}` | 著者アーカイブ |
| GET | `/pages/{slug}` | 固定ページ |
| GET | `/search?q=…` | 全文検索 |
| GET | `/feed` | RSS 2.0 フィード |
| GET | `/feed/atom` | Atom フィード |
| GET | `/sitemap.xml` | XML サイトマップ |
| GET | `/robots.txt` | robots.txt (管理画面で編集可) |
| GET | `/health` | ヘルスチェック (`{"status":"ok"}`) |

### 認証

| メソッド | パス | 説明 |
|--------|------|-------------|
| POST | `/auth/login` | ログイン → JWT 発行 |
| POST | `/auth/register` | ユーザー登録 → JWT 発行 |

### 管理画面 (ログイン必須)

| エリア | パス |
|------|-------|
| ダッシュボード | `GET /admin` (クイックドラフト `POST` も兼用) |
| 投稿 | `/admin/posts` CRUD + バルク操作・公開切り替え・リビジョン |
| 固定ページ | `/admin/pages` CRUD |
| カテゴリ | `/admin/categories` CRUD + 一括削除 |
| タグ | `/admin/tags` + 一括削除 |
| メディア | `/admin/media` アップロード/削除 (画像バリアント自動生成) |
| コメント | `/admin/comments` 承認/拒否/スパム |
| ユーザー | `/admin/users` + ロール管理 |
| メニュー | `/admin/menus` CRUD + ドラッグ&ドロップアイテム管理 |
| ウィジェット | `/admin/widgets` CRUD + 並び替え |
| 設定 | `/admin/settings` + robots.txt 編集 |
| アクティビティログ | `GET /admin/activity-logs` |
| インポート | `GET/POST /admin/import` (WXR) |
| エクスポート | `GET /admin/export` → JSON / WXR / SQLite バックアップ |

### REST API

| メソッド | パス | 説明 |
|--------|------|-------------|
| GET | `/api/posts` | 投稿一覧 (JSON) |
| GET | `/api/posts/{id}` | 個別投稿 (JSON) |
| POST | `/api/posts` | 投稿作成 *(要認証)* |
| PUT | `/api/posts/{id}` | 投稿更新 *(要認証)* |
| DELETE | `/api/posts/{id}` | 投稿削除 *(要認証)* |
| GET | `/api/users/{id}/posts` | ユーザーの投稿一覧 |

## 📊 実装状況

| 機能 | 状態 |
|---------|--------|
| 投稿 CRUD + Markdown レンダリング | ✅ |
| カテゴリ・タグ管理 | ✅ |
| メディアアップロード・ライブラリ | ✅ |
| 画像リサイズ + WebP バリアント + `<img srcset>` | ✅ |
| JWT 認証 + ロールベース権限 | ✅ |
| REST API | ✅ |
| FTS5 全文検索 | ✅ |
| サイト設定 | ✅ |
| Docker / ヘルスチェック | ✅ |
| Web インストーラー | ✅ |
| 固定ページ (Pages) | ✅ |
| アイキャッチ画像 + OGP | ✅ |
| カスタムフィールド (Post Meta) | ✅ |
| 投稿スケジュール | ✅ |
| 投稿リビジョン | ✅ |
| スティッキー投稿 | ✅ |
| テンプレートシステム (テーマ) | ✅ |
| フックシステム (アクション / フィルター) | ✅ |
| ショートコード API | ✅ |
| カスタムナビゲーションメニュー | ✅ |
| パンくずリスト + JSON-LD | ✅ |
| ウィジェットエリア | ✅ |
| ユーザーロール・プロフィール | ✅ |
| 著者アーカイブページ | ✅ |
| コメントシステム (スレッド) | ✅ |
| コメントモデレーション | ✅ |
| RSS 2.0 / Atom フィード | ✅ |
| XML サイトマップ | ✅ |
| SEO メタ + Open Graph + Twitter Card | ✅ |
| 構造化データ (JSON-LD Article) | ✅ |
| バルク操作 (投稿・カテゴリ・タグ) | ✅ |
| アクティビティログ | ✅ |
| ダッシュボード強化 (統計・クイックドラフト・サイトヘルス) | ✅ |
| インメモリページキャッシュ + ETag / Last-Modified | ✅ |
| WordPress WXR インポート (重複チェック・スラッグリネーム) | ✅ |
| JSON エクスポート / WXR エクスポート / SQLite バックアップ | ✅ |

## 🤖 Claude Code スキル

このプロジェクトには AI 支援開発用のカスタムコマンドが含まれています。

### `/wp-implement [タスク名またはフェーズ]`

`ai_docs/wordpress-parity-tasks.md` からタスクを選んで実装。`cargo build` 確認とチェックマーク更新まで自動。

```
/wp-implement フェーズ 8
/wp-implement RSS フィード
/wp-implement              ← 引数なしで未完了タスク一覧を表示
```

### `/wp-add-task [説明]`

ロードマップファイルに新しいタスクを追記します。

## 🧪 テスト

```bash
cargo test
```

統合テスト: `tests/auth_test.rs` · `tests/admin_posts_test.rs` · `tests/public_posts_test.rs` · `tests/api_test.rs` · `tests/taxonomy_test.rs`

## 🚢 デプロイ

### VPS (systemd)

```bash
cargo build --release
# バイナリ + static/ を /opt/redleaf にコピー
# /etc/systemd/system/redleaf.service に JWT_SECRET 等の環境変数を設定
# Nginx でリバースプロキシ + TLS 終端
```

### Fly.io (SQLite に最適な PaaS)

```bash
fly launch --no-deploy
fly volumes create redleaf_data --size 1
fly volumes create redleaf_uploads --size 5
fly secrets set JWT_SECRET="$(openssl rand -hex 32)"
fly deploy
# https://your-app.fly.dev/setup でセットアップ
```

### バックアップ

**管理画面 → Export → Download DB** からいつでも SQLite スナップショットをダウンロードできます。

## 🪄 フィロソフィー

> "RedLeaf — grows naturally, powered by Rust."

Every page is a leaf. Every site is a tree. And Rust is the root that keeps it strong.

## 📜 ライセンス

MIT