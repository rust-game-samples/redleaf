# RedLeaf CMS — WordPress Parity Tasks

WordPress に近づけるための機能追加タスク。
`/wp-add-task` でタスク追加、`/wp-implement` で実装。

---

## 実装済みの WordPress 相当機能 ✅

- 投稿の作成・編集・削除・公開/下書き切り替え
- Markdown レンダリング
- カテゴリ・タグ管理
- スラッグ URL
- ページネーション
- メディアアップロード・ライブラリ
- JWT 認証（ログイン・ログアウト）
- REST API (投稿 CRUD + 認証)
- サイト名・説明・ロゴ設定
- FTS5 全文検索
- Docker 配布
- ヘルスチェックエンドポイント
- Web ベースセットアップウィザード（初回インストーラー）

---

## フェーズ 1 — コンテンツ拡張 ✅ 完了

### 固定ページ (Pages)

- [x] `pages` テーブル追加（投稿とは別エンティティ: title / slug / content / template / status）
- [x] `GET /[slug]` — 固定ページの公開表示
- [x] 管理画面: 固定ページ一覧 (`/admin/pages`)
- [x] 管理画面: 固定ページ作成・編集・削除フォーム
- [x] 固定ページの親子階層 (`parent_id` カラム)
- [ ] フロントページ設定（ホームページに固定ページを表示するオプション）

### アイキャッチ画像 (Featured Image)

- [x] `posts.featured_image_id` カラム追加（`media` テーブルへの外部キー）
- [x] 投稿フォームにアイキャッチ画像選択 UI 追加（メディアライブラリから選択）
- [x] 投稿一覧・投稿詳細テンプレートにアイキャッチ画像表示
- [x] OGP `og:image` にアイキャッチ画像を自動設定

### カスタムフィールド (Post Meta)

- [x] `post_meta` テーブル追加（`post_id` / `key` / `value` KV ストア）
- [x] 管理画面: 投稿フォームにカスタムフィールド追加・編集 UI
- [x] `PostMeta::get(post_id, key)` / `set` / `delete` モデルメソッド
- [ ] REST API でカスタムフィールドを返す (`meta` フィールド)

### 投稿スケジュール (Scheduled Posts)

- [x] `posts.scheduled_at` カラム追加
- [x] 投稿ステータスに `scheduled` を追加（`published=false + scheduled_at IS NOT NULL`）
- [x] バックグラウンドタスク: 予約時刻を過ぎた投稿を自動公開
- [x] 管理画面フォームに日時ピッカーで公開予約 UI 追加

### 投稿リビジョン (Post Revisions)

- [x] `post_revisions` テーブル追加（`post_id` / `title` / `content` / `created_at` / `created_by`）
- [x] 投稿更新時に自動でリビジョン保存（最大 10 件保持）
- [x] 管理画面: リビジョン一覧・ロールバック機能

### スティッキー投稿 (Sticky Posts)

- [x] `posts.sticky` カラム追加（boolean）
- [x] 公開一覧でスティッキー投稿を先頭に固定表示
- [x] 管理画面: 投稿フォームにスティッキー切り替えチェックボックス

---

## フェーズ 2 — テンプレートシステム 🟠 中優先度

### 命名規則

- [ ] テンプレートシステム全体で `wp_` プリフィックスの代わりに `rl_` プリフィックスを使用する（例: `rl_head()` / `rl_footer()` / `rl_enqueue_script()` / `rl_nav_menu()` など）

### `rl_head` / `rl_footer` 相当のフック

- [x] `HookRegistry` 構造体の実装（`add_action` / `do_action` / `add_filter` / `apply_filters`）
- [x] `{% block rl_head %}{% endblock %}` — Askama テンプレートブロック（head タグ内スクリプト/スタイル注入）
- [x] `{% block rl_footer %}{% endblock %}` 相当（フッタースクリプト注入）
- [x] `rl_enqueue_script` / `rl_enqueue_style` 相当の Asset 管理（`src/assets.rs` グローバルシングルトン）

### テンプレートタグ

- [x] `{{ self.the_title() }}` — 投稿タイトル（HTML エスケープ済み）
- [x] `{{ self.the_content()|safe }}` — Markdown → HTML レンダリング済み本文
- [x] `{{ self.the_excerpt() }}` — 抜粋（なければ本文の先頭を自動生成）
- [x] `{{ self.the_permalink() }}` — 正規 URL（post_url_type 設定に従う）
- [x] `{{ self.the_date("%Y-%m-%d") }}` / `{{ post.created_at | rl_date("%B %d, %Y") }}` — 公開日フォーマット
- [x] `{{ self.the_author() }}` — 著者名
- [x] `{{ self.the_post_thumbnail("medium")|safe }}` — アイキャッチ画像 `<img>` タグ
- [x] `{{ self.site_url() }}` / `{{ self.home_url() }}` — サイト URL

### テンプレート階層 (Template Hierarchy)

- [x] テーマディレクトリ構造の設計（`templates/themes/default/` 以下）
- [x] `single.html` / `archive.html` / `page.html` / `home.html` / `404.html` の WordPress 命名規則テンプレート（ルートハンドラで自動選択）
- [x] `{% include "themes/default/partials/pagination.html" %}` — `get_template_part` 相当のテンプレートインクルード
- [x] `active_theme` 設定（DB 保存・管理画面設定可能。ランタイム切り替えは将来の Tera 対応時に実装）

### ウィジェットエリア

- [x] `widget_areas` / `widgets` テーブル設計
- [x] ビルトインウィジェット: 最近の投稿・カテゴリ一覧・タグクラウド・テキスト・検索フォーム
- [x] 管理画面: ウィジェット管理 UI（ドラッグ＆ドロップ配置）
- [x] テンプレート内での `{{ render_widget_area("sidebar") }}` 呼び出し

---

## フェーズ 3 — ナビゲーション 🟠 中優先度

### カスタムメニュー

- [x] `nav_menus` / `nav_menu_items` テーブル追加
- [x] 管理画面: メニュー作成・アイテム追加（URL / 固定ページ / カテゴリ / 外部リンク）
- [x] 管理画面: ドラッグ＆ドロップでメニュー並び替え・階層設定
- [x] テンプレート内 `{{ rl_nav_menu(location) }}` で `<ul>` レンダリング
- [x] メニューロケーション登録（primary / footer / social など）

### パンくずリスト (Breadcrumbs)

- [x] `Breadcrumb` 構造体の設計（ページ種別に応じた自動生成）
- [x] テンプレートタグ `{{ the_breadcrumb() }}` で `<nav aria-label="breadcrumb">` を出力
- [x] 構造化データ (JSON-LD `BreadcrumbList`) の自動出力

---

## フェーズ 4 — ユーザー・ロール 🟠 中優先度

### ユーザーロールと権限

- [x] `roles` / `capabilities` テーブル設計（Administrator / Editor / Author / Contributor / Subscriber）
- [x] `users.role` カラム追加
- [x] 権限チェックミドルウェア (`require_capability("edit_posts")`)
- [x] 管理画面: ユーザー一覧・ロール変更 UI

### ユーザープロフィール

- [x] `users` テーブルに `display_name` / `bio` / `website` / `avatar_url` カラム追加
- [x] 管理画面: プロフィール編集ページ (`/admin/profile`)
- [x] パスワード変更フォーム

### 著者アーカイブページ

- [x] `GET /author/{username}` — 著者の公開投稿一覧ページ
- [x] 著者プロフィールカード（bio / アバター）の表示
- [x] REST API: `GET /api/users/{id}/posts`

---

## フェーズ 5 — コメント 🟡 低優先度

### コメントシステム

- [x] `comments` テーブル追加（`post_id` / `author_name` / `author_email` / `content` / `approved` / `parent_id`）
- [x] 公開投稿ページにコメント投稿フォーム追加
- [x] スレッドコメント（親子関係の再帰表示）
- [x] コメント数の投稿表示への反映

### コメントモデレーション

- [x] 管理画面: コメント一覧・承認/拒否/スパム操作 (`/admin/comments`)
- [ ] 新規コメント通知メール（SMTP 設定）
- [ ] Akismet 互換スパムフィルター API 連携

---

## フェーズ 6 — SEO・フィード 🟠 中優先度

### RSS / Atom フィード

- [x] `GET /feed` — RSS 2.0 フィード（最新20件）
- [x] `GET /feed/atom` — Atom フィード
- [x] カテゴリ別フィード (`/category/{slug}/feed`)
- [x] `<link rel="alternate">` タグをテンプレート head に自動出力

### XML サイトマップ

- [x] `GET /sitemap.xml` — 全公開投稿・固定ページの URL 一覧
- [x] サイトマップインデックス (`/sitemap_index.xml`) で分割対応
- [x] `<lastmod>` / `<changefreq>` / `<priority>` 設定

### OGP / SEO メタタグ

- [x] `posts.seo_title` / `posts.seo_description` カラム追加
- [x] テンプレート `<head>` に Open Graph タグ自動出力（`og:title` / `og:description` / `og:image`）
- [x] Twitter Card メタタグ出力
- [x] Canonical URL `<link rel="canonical">` 出力
- [x] 構造化データ JSON-LD（`Article` スキーマ）の自動出力

### robots.txt

- [x] `GET /robots.txt` — 動的生成（サイトマップ URL 含む）
- [x] 管理画面: robots.txt 内容の編集 UI

---

## フェーズ 7 — プラグイン・拡張 🔵 将来拡張

### フックシステム (Actions / Filters)

- [x] `ActionRegistry` — `do_action(hook, ...)` / `add_action(hook, callback, priority)`
- [x] `FilterRegistry` — `apply_filters(hook, value)` / `add_filter(hook, callback, priority)`
- [x] ビルトインフック: `before_post_save` / `after_post_publish` / `on_user_login`（各ハンドラに配線済み）
- [ ] WASM プラグインランタイム（将来的な外部プラグイン対応）

### ショートコード API

- [x] `ShortcodeRegistry` — `[gallery]` / `[caption]` / `[audio]` などの解析・実行（`src/shortcodes.rs`）
- [x] ビルトインショートコード: `[gallery ids="1,2,3"]` / `[caption]` / `[audio src="..."]`
- [x] コンテンツレンダリング時に自動展開（`markdown_to_html` パイプライン統合）

### カスタム投稿タイプ (Custom Post Types)

- [ ] `post_types` テーブル / JSON 設定ファイルによる定義
- [ ] 自動ルーティング生成（一覧・詳細・管理画面 CRUD）
- [ ] REST API エンドポイント自動生成
- [ ] カスタム投稿タイプ用アーカイブページ

---

## フェーズ 8 — 管理画面強化 🟠 中優先度

### リッチテキストエディタ

- [x] 管理画面投稿フォームに [EasyMDE](https://github.com/Ionaru/easy-markdown-editor) を統合
- [x] 画像挿入ボタン（メディアライブラリ連携）
- [x] エディタのプレビューモード（リアルタイム Markdown プレビュー）
- [x] フルスクリーンエディタモード

### バルク操作

- [x] 投稿一覧にチェックボックス追加
- [x] バルク操作メニュー: 一括削除 / 一括公開 / 一括非公開
- [x] カテゴリ・タグの一括削除

### アクティビティログ

- [x] `activity_logs` テーブル（`user_id` / `action` / `target_type` / `target_id` / `created_at`）
- [x] 投稿作成・更新・削除・ログイン等のイベント記録
- [x] 管理画面: アクティビティログ表示ページ

### ダッシュボード強化

- [x] ダッシュボードに最近のコメント・メディア数・ユーザー数ウィジェット追加
- [x] クイックドラフト（ダッシュボードから即座に下書き作成）
- [x] サイトヘルス情報（PHP バージョン相当の Rust バージョン・DB サイズなど）

---

## フェーズ 9 — パフォーマンス 🔵 将来拡張

### 画像処理

- [x] アップロード時の画像リサイズ（thumbnail / medium / large サイズ自動生成）
- [x] `image` crate を使ったサーバーサイド画像処理
- [x] WebP 変換対応
- [x] `<img srcset>` の自動生成

### キャッシュ

- [x] ページキャッシュレイヤー（`tower` ミドルウェア）
- [x] 投稿更新時のキャッシュ自動パージ
- [x] ETag / Last-Modified ヘッダー対応

---

## フェーズ 10 — インポート・エクスポート 🔵 将来拡張

### WordPress インポート

- [ ] WordPress WXR (XML) ファイルのパース
- [ ] 投稿・ページ・カテゴリ・タグ・メディアのインポート
- [ ] 重複チェック・スラッグ衝突解決
- [ ] インポート進捗表示 UI

### エクスポート・バックアップ

- [ ] `GET /admin/export` — 全コンテンツを JSON でエクスポート
- [ ] WordPress 互換 WXR 形式でのエクスポート
- [ ] SQLite DB ファイルのバックアップダウンロード

---

## 追加タスクのメモ欄

> `/wp-add-task <説明>` コマンドで追記。