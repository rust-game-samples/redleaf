# RedLeaf CMS — アーキテクチャ

最終更新: 2026-05-29

## 概要

RedLeaf は Rust 製の軽量 CMS。WordPress ライクなブログ管理システムを目指し、パフォーマンス・シンプルさ・API ファーストを設計方針とする。

---

## レイヤー構成

```
┌──────────────────────────────────────────────────────────┐
│  クライアント (ブラウザ / REST クライアント)                 │
└─────────────────────────┬────────────────────────────────┘
                          │ HTTP
┌─────────────────────────▼────────────────────────────────┐
│  Middleware Layer                                         │
│  ・PageCache (Tower from_fn) — 公開 GET をキャッシュ      │
│  ・Extension<Arc<PageCache>> — ハンドラからパージ可能      │
│  ・require_auth (管理画面保護)                             │
│  ・TraceLayer (アクセスログ)                               │
│  ・ServeDir /static/* (静的ファイル直接配信)               │
└─────────────────────────┬────────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────────┐
│  Route Layer (routes/)                                    │
│  ・mod.rs      公開ページ・サイトマップ・ロボット           │
│  ・posts.rs    投稿表示 (Markdown→HTML + srcset)          │
│  ・taxonomy.rs カテゴリ/タグアーカイブ                     │
│  ・feed.rs     RSS 2.0 / Atom フィード                    │
│  ・admin.rs    管理画面 全ハンドラ (2500+ 行)              │
│  ・api.rs      REST API /api/posts                        │
│  ・auth.rs     /auth/login, /auth/register                │
└─────────────────────────┬────────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────────┐
│  Model Layer (models/)                                    │
│  ・各モデルが自身の DB 操作メソッドを保有                   │
│  ・SQLx FromRow で SQL 結果を構造体へ直接マッピング         │
│  ・HookRegistry (before_post_save / after_post_publish 等) │
└─────────────────────────┬────────────────────────────────┘
                          │ SQLx (非同期プール)
┌─────────────────────────▼────────────────────────────────┐
│  SQLite (redleaf.db)                                      │
│  ・15 マイグレーション (バイナリ埋め込み・自動適用)          │
│  ・FTS5 仮想テーブルで全文検索                              │
└──────────────────────────────────────────────────────────┘
```

---

## リクエストフロー

### 公開投稿ページ表示

```
GET /posts/my-post-slug
  → PageCache ミドルウェア
    ├─ HIT → キャッシュから即返却 (ETag / 304 対応)
    └─ MISS →
         show_post(Path, State<DbPool>)
           ├─ Post::find_by_slug_with_author(&pool, slug)
           ├─ Tag::find_by_post(&pool, post.id)
           ├─ Comment::find_by_post(&pool, post.id)
           ├─ Media::find_variants(&pool, featured_image_id)   ← 画像 srcset
           ├─ markdown_to_html(&post.content)                  ← Markdown + ショートコード展開
           └─ Askama PostShowTemplate → HTML レスポンス
                → キャッシュに保存 (TTL 5分)
```

### 投稿保存 (管理画面)

```
POST /admin/posts/{id}
  → require_auth ミドルウェア (JWT 検証 → Claims extension)
  → update_post(State<DbPool>, Extension<Claims>, Extension<PageCache>, Path<id>, Form<PostForm>)
       ├─ PostRevision::save(&pool, ...)      ← 現バージョンをリビジョン保存
       ├─ Post::update(&pool, id, payload)
       ├─ Post::set_tags(&pool, post.id, &tag_ids)
       ├─ PostMeta::replace_all(&pool, post.id, &pairs)
       ├─ ActivityLog::create(&pool, ...)     ← 操作を記録
       ├─ cache.invalidate_all()              ← ページキャッシュ全クリア
       └─ Redirect::to("/admin/posts")
```

### 画像アップロード

```
POST /admin/media/upload (multipart)
  → upload_media handler
       ├─ tokio::fs::write(path, bytes)       ← オリジナル保存
       ├─ Media::create(&pool, ...)           ← DB 登録
       └─ media.is_image() == true なら:
            tokio::task::spawn_blocking(|| {
              image_processing::generate_variants(bytes, mime, stem)
                → thumbnail (150×150 crop) + medium (300px) + large (1024px)
                → 各サイズの WebP バリアント
            })
            → variants: Vec<VariantInfo>
            → 各ファイルを static/uploads/ に書き出し
            → Media::create_variant(&pool, ...) ×n
```

---

## 状態管理

| 状態 | 保持場所 | アクセス方法 |
|---|---|---|
| DB コネクションプール | `Router::with_state(pool)` | `State(pool): State<DbPool>` |
| ページキャッシュ | `Arc<PageCache>` (アプリ起動時生成) | `Extension(cache): Extension<Arc<PageCache>>` |
| ログインユーザー | JWT Cookie → ミドルウェアが検証 | `Extension(claims): Extension<Claims>` |
| サイト設定 | DB `settings` テーブル (KV) | `Setting::get(&pool, key).await` |

---

## セキュリティ設計

| 項目 | 実装 |
|---|---|
| パスワード保存 | Argon2id (メモリハード / PHC 文字列形式) |
| SQL インジェクション | SQLx プリペアドステートメント |
| JWT 認証 | HS256 / HttpOnly Cookie / SameSite=Strict / 7日有効期限 |
| 管理画面保護 | `require_auth` ミドルウェア (全管理ルートに適用) |
| 権限チェック | `has_capability(role, cap)` で細粒度制御 |
| XSS 対策 | Askama テンプレートが自動エスケープ / `|safe` フィルターは明示的に記述 |
| ファイルアップロード | MIME タイプチェック / ランダムファイル名生成 |

---

## 拡張ポイント

| 機能 | 拡張方法 |
|---|---|
| テーマ | `templates/themes/{name}/` にテンプレートを追加、管理画面の `active_theme` で切り替え |
| ショートコード | `ShortcodeRegistry::register(name, handler)` で登録 |
| フック | `hooks::add_action(name, callback)` / `add_filter(name, callback)` |
| REST API | `src/routes/api.rs` にエンドポイント追加 |
| DB (将来) | SQLx の `DATABASE_URL` を変更すれば PostgreSQL 等に移行可 (要クエリ互換確認) |