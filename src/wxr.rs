//! WordPress WXR (eXtended RSS) 1.2 parser.
//!
//! Handles the namespaced elements used in WordPress exports:
//! `wp:`, `dc:`, `content:`, `excerpt:`.

use quick_xml::events::Event;
use quick_xml::Reader;

// ─── Output types ─────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct WxrData {
    pub site_title: String,
    pub categories: Vec<WxrCategory>,
    pub tags: Vec<WxrTag>,
    pub posts: Vec<WxrItem>,
    pub pages: Vec<WxrItem>,
}

#[derive(Debug, Default)]
pub struct WxrCategory {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Default)]
pub struct WxrTag {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Default)]
pub struct WxrItem {
    pub title: String,
    pub creator: String,
    pub content: String,
    pub excerpt: String,
    /// "YYYY-MM-DD HH:MM:SS" or empty.
    pub post_date: String,
    pub slug: String,
    /// "publish" | "draft" | "private" etc.
    pub status: String,
    /// "post" | "page" | "attachment" etc.
    pub post_type: String,
    pub is_sticky: bool,
    pub parent_id: i64,
    /// Category slugs linked to this item.
    pub cat_slugs: Vec<String>,
    /// Tag slugs linked to this item.
    pub tag_slugs: Vec<String>,
    pub meta: Vec<(String, String)>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(xml_bytes: &[u8]) -> Result<WxrData, String> {
    let xml_str = std::str::from_utf8(xml_bytes)
        .map_err(|e| format!("Invalid UTF-8: {e}"))?;

    let mut reader = Reader::from_str(xml_str);
    reader.config_mut().trim_text(true);

    let mut data = WxrData::default();

    // ── State flags ──
    let mut in_item = false;
    let mut in_wp_category = false; // channel-level <wp:category>
    let mut in_wp_tag = false;
    let mut in_wp_postmeta = false;

    // Tracks which element we are currently reading text for.
    let mut cur_elem = String::new();

    // Per-entity accumulators.
    let mut cur_cat = WxrCategory::default();
    let mut cur_tag = WxrTag::default();
    let mut cur_item = WxrItem::default();
    let mut meta_key = String::new();
    let mut meta_value = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = qname_str(e.name().as_ref());
                match name.as_str() {
                    "item" => {
                        in_item = true;
                        cur_item = WxrItem::default();
                    }
                    "wp:category" if !in_item => {
                        in_wp_category = true;
                        cur_cat = WxrCategory::default();
                    }
                    "wp:tag" => {
                        in_wp_tag = true;
                        cur_tag = WxrTag::default();
                    }
                    "wp:postmeta" if in_item => {
                        in_wp_postmeta = true;
                        meta_key.clear();
                        meta_value.clear();
                    }
                    // Item-level category/tag reference (no namespace prefix).
                    "category" if in_item => {
                        let domain = find_attr(&e, b"domain");
                        let nicename = find_attr(&e, b"nicename");
                        match domain.as_str() {
                            "category" => cur_item.cat_slugs.push(nicename),
                            "post_tag" => cur_item.tag_slugs.push(nicename),
                            _ => {}
                        }
                    }
                    _ => {}
                }
                cur_elem = name;
            }

            Ok(Event::End(e)) => {
                let name = qname_str(e.name().as_ref());
                match name.as_str() {
                    "item" => {
                        let item = std::mem::take(&mut cur_item);
                        match item.post_type.as_str() {
                            "page" => data.pages.push(item),
                            // Treat empty or unknown post_type as "post".
                            _ => data.posts.push(item),
                        }
                        in_item = false;
                    }
                    "wp:category" if in_wp_category => {
                        data.categories.push(std::mem::take(&mut cur_cat));
                        in_wp_category = false;
                    }
                    "wp:tag" if in_wp_tag => {
                        data.tags.push(std::mem::take(&mut cur_tag));
                        in_wp_tag = false;
                    }
                    "wp:postmeta" if in_wp_postmeta => {
                        if !meta_key.is_empty() {
                            cur_item.meta.push((
                                std::mem::take(&mut meta_key),
                                std::mem::take(&mut meta_value),
                            ));
                        }
                        in_wp_postmeta = false;
                    }
                    _ => {}
                }
                cur_elem.clear();
            }

            Ok(Event::Text(e)) => {
                let text = e.unescape().map(|s| s.into_owned()).unwrap_or_default();
                apply_text(
                    text, &cur_elem,
                    in_item, in_wp_category, in_wp_tag, in_wp_postmeta,
                    &mut cur_cat, &mut cur_tag, &mut cur_item,
                    &mut meta_key, &mut meta_value, &mut data,
                );
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).into_owned();
                apply_text(
                    text, &cur_elem,
                    in_item, in_wp_category, in_wp_tag, in_wp_postmeta,
                    &mut cur_cat, &mut cur_tag, &mut cur_item,
                    &mut meta_key, &mut meta_value, &mut data,
                );
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
    }

    Ok(data)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn qname_str(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).into_owned()
}

fn find_attr(e: &quick_xml::events::BytesStart, attr_name: &[u8]) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == attr_name)
        .map(|a| String::from_utf8_lossy(a.value.as_ref()).into_owned())
        .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn apply_text(
    text: String,
    cur_elem: &str,
    in_item: bool,
    in_wp_category: bool,
    in_wp_tag: bool,
    in_wp_postmeta: bool,
    cur_cat: &mut WxrCategory,
    cur_tag: &mut WxrTag,
    cur_item: &mut WxrItem,
    meta_key: &mut String,
    meta_value: &mut String,
    data: &mut WxrData,
) {
    if text.is_empty() {
        return;
    }

    if in_wp_category {
        match cur_elem {
            "wp:cat_name" => cur_cat.name = text,
            "wp:category_nicename" => cur_cat.slug = text,
            _ => {}
        }
    } else if in_wp_tag {
        match cur_elem {
            "wp:tag_name" => cur_tag.name = text,
            "wp:tag_slug" => cur_tag.slug = text,
            _ => {}
        }
    } else if in_item {
        if in_wp_postmeta {
            match cur_elem {
                "wp:meta_key" => *meta_key = text,
                "wp:meta_value" => *meta_value = text,
                _ => {}
            }
        } else {
            match cur_elem {
                "title" => cur_item.title = text,
                "dc:creator" => cur_item.creator = text,
                "content:encoded" => cur_item.content = text,
                "excerpt:encoded" => cur_item.excerpt = text,
                "wp:post_name" => cur_item.slug = text,
                "wp:status" => cur_item.status = text,
                "wp:post_type" => cur_item.post_type = text,
                "wp:post_date" | "wp:post_date_gmt" => {
                    if cur_item.post_date.is_empty()
                        || cur_item.post_date == "0000-00-00 00:00:00"
                    {
                        cur_item.post_date = text;
                    }
                }
                "wp:is_sticky" => cur_item.is_sticky = text == "1",
                "wp:post_parent" => cur_item.parent_id = text.parse().unwrap_or(0),
                _ => {}
            }
        }
    } else if cur_elem == "title" && data.site_title.is_empty() {
        data.site_title = text;
    }
}