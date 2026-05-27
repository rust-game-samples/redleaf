# RedLeaf CMS - 実装タスク

## 現状サマリー

フェーズ 1 の基盤部分（DB・モデル・認証スキャフォールド）は完成。  
管理画面 UI・JWT 発行・API エンドポイントが未実装。

---

## 実装済み ✅

### 基盤

- [x] Axum サーバー起動・ルーティング設定 (`main.rs`)
- [x] SQLite コネクションプール初期化 (`db.rs`)
- [x] `sqlx::migrate!` による自動スキーマ適用
- [x] `.env` による設定外部化（dotenvy）
- [x] tracing ログ基盤

### データベース

- [x] `users` テーブル + インデックス（email・username）
- [x] `posts` テーブル + インデックス（published・slug）

### ユーザー認証モデル

- [x] `User` / `CreateUser` / `LoginUser` 構造体定義
- [x] Argon2id パスワードハッシュ・検証
- [x] ユーザー作成 (`User::create`)
- [x] メール・ユーザー名・ID による検索
- [x] ログイン認証 (`User::authenticate`)

### 投稿モデル

- [x] `Post` / `CreatePost` / `UpdatePost` 構造体定義
- [x] 投稿一覧取得（公開済み・新着順）
- [x] ID / スラッグ による単一投稿取得
- [x] 投稿作成・更新（部分更新対応）・削除

### 公開フロントエンド

- [x] ホームページ (`/`)
- [x] 投稿一覧ページ (`/posts/`)
- [x] 個別投稿ページ (`/posts/{id}`)
- [x] Markdown → HTML レンダリング（strikethrough・tables 有効）
- [x] 静的ファイル配信 (`/static/*`)

### 管理画面

- [x] 管理ダッシュボード骨格 (`/admin/`)

---

## 未実装 / TODO

### 優先度: 高（フェーズ 1 完成に必要）

#### JWT 認証

- [x] `Claims` 構造体定義（`src/auth.rs`）
- [x] `POST /auth/register` — ユーザー登録 → JWT 発行（201 Created）
- [x] `POST /auth/login` — ログイン → JWT 発行（200 / 401）
- [x] JWT 検証ミドルウェア（`src/middleware.rs`、`from_fn` で実装）
- [x] 管理画面ルートへの認証ミドルウェア適用（`/admin` は Bearer トークン必須）

#### 管理画面 CRUD

- [x] `GET  /admin/posts` — 投稿一覧（全ステータス・公開/下書きバッジ付き）
- [x] `GET  /admin/posts/new` — 投稿作成フォーム（スラッグ自動生成 JS 付き）
- [x] `POST /admin/posts` — 投稿作成処理（303 リダイレクト）
- [x] `GET  /admin/posts/{id}/edit` — 投稿編集フォーム（既存値プリフィル）
- [x] `POST /admin/posts/{id}` — 投稿更新処理
- [x] `POST /admin/posts/{id}/delete` — 投稿削除処理（確認ダイアログ付き）
- [x] `POST /admin/posts/{id}/toggle` — 公開/下書き切り替え
- [x] ダッシュボードに投稿数統計（Total / Published / Drafts）

#### テンプレート移行

- [x] `templates/` ディレクトリ作成（`posts/`・`admin/posts/` サブディレクトリ含む）
- [x] `base.html`（公開ページ共通レイアウト）
- [x] `index.html`（ホームページ）
- [x] `posts/list.html`（投稿一覧・excerpt 表示）
- [x] `posts/show.html`（投稿詳細・Markdown レンダリング）
- [x] `admin/base.html`（管理ページ共通レイアウト・ナビゲーション）
- [x] `admin/dashboard.html`（統計ダッシュボード）
- [x] `admin/posts/list.html`（投稿一覧・バッジ・アクション）
- [x] `admin/posts/form.html`（作成・編集共通フォーム・エラー表示・スラッグ自動生成 JS）
- [x] `src/util.rs` に `render<T: Template>()` ヘルパー追加（askama_axum バージョン非互換を回避）

---

### 優先度: 中（フェーズ 2）

#### 投稿スラッグ URL 対応

- [ ] 個別投稿を `/posts/{id}` から `/posts/{slug}` へ変更
- [ ] スラッグ自動生成（タイトルから ASCII 変換）

#### 投稿者

- [ ] `posts.author_id` カラム追加（マイグレーション）
- [ ] `Post` 構造体に `author_id: Option<i64>` 追加
- [ ] 投稿作成時に認証ユーザーの ID を自動セット

#### ページネーション

- [ ] 投稿一覧に `LIMIT` / `OFFSET` 追加
- [ ] ページネーション UI コンポーネント

#### エラーハンドリング統一

- [ ] `src/errors.rs` に統一エラー型定義（`thiserror` 使用）
- [ ] HTTP ステータスコードへのマッピング実装
- [ ] 各ハンドラのエラー処理を統一型へ移行

---

### 優先度: 低（将来拡張）

#### REST API

- [ ] `GET    /api/posts` — 投稿一覧 JSON
- [ ] `GET    /api/posts/{id}` — 個別投稿 JSON
- [ ] `POST   /api/posts` — 投稿作成（認証必須）
- [ ] `PUT    /api/posts/{id}` — 投稿更新（認証必須）
- [ ] `DELETE /api/posts/{id}` — 投稿削除（認証必須）
- [ ] API キー認証 or JWT Bearer

#### カテゴリ・タグ

- [ ] `categories` / `tags` / `post_tags` テーブル追加
- [ ] 投稿作成フォームにカテゴリ・タグ選択
- [ ] カテゴリ別・タグ別一覧ページ

#### メディア管理

- [ ] `media` テーブル追加
- [ ] ファイルアップロード処理（`multipart/form-data`）
- [ ] `./static/uploads/` への保存
- [ ] メディアライブラリ画面

#### サイト設定

- [ ] `settings` テーブル（KV ストア）
- [ ] サイト名・説明・ロゴ URL の管理
- [ ] 設定画面 UI

#### システム

- [ ] Dockerfile 作成
- [ ] ヘルスチェックエンドポイント (`GET /health`)
- [ ] ページ全文検索（SQLite FTS5）
- [ ] GUI インストーラー

---

## 実装順序の推奨

```
1. JWT 認証エンドポイント + ミドルウェア
   └→ 管理画面を安全に保護する基盤

2. 管理画面 投稿 CRUD（フォーム実装）
   └→ テンプレート移行と同時に進める

3. Askama テンプレート移行
   └→ 管理画面 CRUD と並行・もしくは先行

4. スラッグ URL 対応 + 投稿者フィールド
   └→ SEO と将来の多ユーザー対応

5. REST API
   └→ ヘッドレス CMS 用途

6. カテゴリ・タグ・メディア管理
   └→ コンテンツの充実
```