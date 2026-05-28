# RedLeaf CMS — ディレクトリ構造

最終更新: 2026-05-29

## ツリー

```
redleaf/
├── src/
│   ├── main.rs                  # エントリポイント・サーバー起動
│   ├── lib.rs                   # build_app() — ルーター・ミドルウェア配線
│   ├── assets.rs                # rl_enqueue_script/style レジストリ
│   ├── auth.rs                  # JWT 生成 (generate_token) / 検証 (Claims)
│   ├── cache.rs                 # インメモリページキャッシュ + Tower ミドルウェア
│   ├── db.rs                    # SqlitePool 初期化 (最大5接続)
│   ├── errors.rs                # AppError (NotFound / Database / Internal)
│   ├── filters.rs               # Askama テンプレートフィルター (rl_date 等)
│   ├── hooks.rs                 # ActionRegistry / FilterRegistry (WordPress 風フック)
│   ├── image_processing.rs      # 画像リサイズ (thumbnail/medium/large) + WebP 生成
│   ├── middleware.rs            # require_auth / has_capability
│   ├── shortcodes.rs            # ShortcodeRegistry ([gallery][caption][audio] 等)
│   ├── util.rs                  # slugify / render / Pagination / markdown_to_html
│   ├── wxr.rs                   # WordPress WXR 1.2 XML パーサー
│   │
│   ├── models/
│   │   ├── mod.rs               # 全モデルの pub use 再エクスポート
│   │   ├── activity_log.rs      # ActivityLog / ActivityLogWithUser
│   │   ├── category.rs          # Category / CategoryWithCount
│   │   ├── comment.rs           # Comment / CommentWithPost (スレッド描画)
│   │   ├── media.rs             # Media / MediaVariant (画像バリアント)
│   │   ├── nav_menu.rs          # NavMenu / NavMenuItem / LOCATIONS
│   │   ├── page.rs              # Page / CreatePage / UpdatePage
│   │   ├── post.rs              # Post / PostWithAuthor / CreatePost / UpdatePost
│   │   ├── post_meta.rs         # PostMeta (KV カスタムフィールド)
│   │   ├── post_revision.rs     # PostRevision (最大10件保持)
│   │   ├── setting.rs           # Setting (サイト設定 KV)
│   │   ├── tag.rs               # Tag / TagWithCount
│   │   ├── user.rs              # User / CreateUser / LoginUser / UpdateProfile
│   │   └── widget.rs            # Widget / WidgetArea
│   │
│   └── routes/
│       ├── mod.rs               # 公開ルート (/, /search, /setup, /sitemap 等)
│       ├── admin.rs             # 管理画面 全ハンドラ (~2500行)
│       ├── api.rs               # REST API /api/posts
│       ├── auth.rs              # /auth/login, /auth/register
│       ├── feed.rs              # /feed (RSS2.0), /feed/atom, /category/.../feed
│       ├── posts.rs             # /posts/{slug} 公開投稿ページ
│       └── taxonomy.rs          # /categories/{slug}, /tags/{slug}
│
├── templates/
│   ├── admin/
│   │   ├── base.html            # 管理画面共通レイアウト (ナビ・CSS)
│   │   ├── dashboard.html       # ダッシュボード (統計・クイックドラフト・サイトヘルス)
│   │   ├── login.html / register.html / profile.html / settings.html
│   │   ├── export.html          # エクスポート (JSON/WXR/DB)
│   │   ├── import.html          # WXR インポート + 結果表示
│   │   ├── activity_logs.html   # アクティビティログ
│   │   ├── posts/               # list.html / form.html / revisions.html
│   │   ├── pages/               # list.html / form.html
│   │   ├── categories/          # list.html / form.html
│   │   ├── tags/                # list.html
│   │   ├── media/               # list.html
│   │   ├── comments/            # list.html
│   │   ├── users/               # list.html
│   │   ├── menus/               # list.html / edit.html
│   │   └── widgets/             # index.html
│   │
│   ├── themes/default/
│   │   ├── single.html          # 投稿詳細 (srcset / OGP / コメント)
│   │   ├── archive.html         # 投稿一覧
│   │   ├── home.html            # ホームページ
│   │   ├── page.html            # 固定ページ
│   │   ├── author.html          # 著者アーカイブ
│   │   ├── search.html          # 検索結果
│   │   ├── 404.html
│   │   ├── partials/            # header / footer / sidebar / pagination 等
│   │   └── taxonomy/            # category.html / tag.html
│   │
│   ├── base.html                # 公開ページ共通ベース
│   ├── index.html               # ホームページ
│   ├── search.html              # 全文検索結果
│   ├── setup.html               # 初回セットアップウィザード
│   ├── posts/                   # 公開投稿 (list / show)
│   └── pages/                   # 固定ページ表示
│
├── migrations/                  # SQLx 自動マイグレーション (バイナリ埋め込み)
│   ├── 20240101000000_initial.sql
│   ├── 20240102000000_create_settings.sql
│   ├── 20240103000000_add_author_to_posts.sql
│   ├── 20240104000000_create_categories_tags.sql
│   ├── 20240105000000_create_media.sql
│   ├── 20240106000000_seed_site_settings.sql
│   ├── 20240107000000_create_fts5.sql
│   ├── 20240108000000_phase1_content.sql    # pages / post_meta / revisions / sticky
│   ├── 20240109000000_widgets.sql
│   ├── 20240110000000_nav_menus.sql
│   ├── 20240111000000_user_roles_profiles.sql
│   ├── 20240112000000_comments.sql
│   ├── 20240113000000_seo_fields.sql
│   ├── 20240114000000_activity_logs.sql
│   └── 20240115000000_image_variants.sql
│
├── static/
│   └── uploads/                 # アップロードファイル + 画像バリアント
│
├── tests/
│   ├── common/                  # テスト共通ヘルパー
│   ├── auth_test.rs
│   ├── admin_posts_test.rs
│   ├── public_posts_test.rs
│   ├── api_test.rs
│   └── taxonomy_test.rs
│
├── ai_docs/                     # 本ドキュメント群
│   ├── architecture.md
│   ├── data-struture.md
│   ├── directory-struture.md    # 本ファイル
│   ├── implementation-tasks.md
│   ├── requirement.md
│   ├── tech-stack.md
│   └── wordpress-parity-tasks.md
│
├── .claude/
│   └── commands/
│       ├── wp-add-task.md       # /wp-add-task スキル定義
│       └── wp-implement.md      # /wp-implement スキル定義
│
├── build.rs                     # RUST_VERSION 環境変数をコンパイル時埋め込み
├── Cargo.toml                   # 依存関係定義
├── Cargo.lock                   # バージョンロック
├── Dockerfile                   # マルチステージビルド
├── redleaf.db                   # SQLite DB (開発用)
└── README.md / README_ja.md
```

---

## ソースコード規模 (主要ファイル)

| ファイル | 行数 (概算) |
|---|---|
| src/routes/admin.rs | ~2,500 |
| src/models/post.rs | ~500 |
| src/routes/mod.rs | ~450 |
| src/routes/posts.rs | ~430 |
| src/models/widget.rs | ~330 |
| src/models/nav_menu.rs | ~280 |
| src/shortcodes.rs | ~260 |
| src/wxr.rs | ~255 |
| src/routes/feed.rs | ~220 |
| src/routes/api.rs | ~215 |
| src/cache.rs | ~200 |
| **src/ 合計** | **~7,800** |

---

## 主要な状態・依存の流れ

```
main.rs
  └─ build_app(pool)                      ← lib.rs
       ├─ Arc<PageCache>                   ← cache.rs (インメモリ)
       ├─ Extension(page_cache)            ← 全ハンドラから利用可能
       ├─ from_fn(cache::middleware)       ← 公開 GET を自動キャッシュ
       ├─ from_fn(middleware::require_auth)← 管理画面保護
       └─ Router::with_state(pool)         ← DbPool を全ハンドラに注入
```