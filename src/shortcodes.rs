use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

type ShortcodeFn = Arc<dyn Fn(&HashMap<String, String>, Option<&str>) -> String + Send + Sync>;

pub struct ShortcodeRegistry {
    handlers: HashMap<String, ShortcodeFn>,
}

impl ShortcodeRegistry {
    fn new() -> Self {
        let mut s = Self { handlers: HashMap::new() };
        register_builtins(&mut s);
        s
    }

    pub fn register(
        &mut self,
        tag: impl Into<String>,
        handler: impl Fn(&HashMap<String, String>, Option<&str>) -> String + Send + Sync + 'static,
    ) {
        self.handlers.insert(tag.into(), Arc::new(handler));
    }

    pub fn expand(&self, content: &str) -> String {
        expand_in(content, &self.handlers)
    }
}

// ─── Global singleton ─────────────────────────────────────────────────────────

static INSTANCE: OnceLock<Mutex<ShortcodeRegistry>> = OnceLock::new();

fn registry() -> &'static Mutex<ShortcodeRegistry> {
    INSTANCE.get_or_init(|| Mutex::new(ShortcodeRegistry::new()))
}

/// Register a custom shortcode handler.
pub fn add_shortcode(
    tag: impl Into<String>,
    handler: impl Fn(&HashMap<String, String>, Option<&str>) -> String + Send + Sync + 'static,
) {
    registry().lock().unwrap().register(tag, handler);
}

/// Expand all shortcodes in `content` and return the processed string.
pub fn expand_shortcodes(content: &str) -> String {
    registry().lock().unwrap().expand(content)
}

// ─── Parser ───────────────────────────────────────────────────────────────────

fn expand_in(content: &str, handlers: &HashMap<String, ShortcodeFn>) -> String {
    let mut result = String::with_capacity(content.len());
    let mut pos = 0;

    while pos < content.len() {
        match content[pos..].find('[') {
            None => {
                result.push_str(&content[pos..]);
                break;
            }
            Some(rel) => {
                let abs = pos + rel;
                result.push_str(&content[pos..abs]);
                match try_parse(content, abs, handlers) {
                    Some((output, end)) => {
                        result.push_str(&output);
                        pos = end;
                    }
                    None => {
                        result.push('[');
                        pos = abs + 1;
                    }
                }
            }
        }
    }
    result
}

fn try_parse(
    content: &str,
    start: usize,
    handlers: &HashMap<String, ShortcodeFn>,
) -> Option<(String, usize)> {
    let inner = &content[start + 1..]; // skip '['

    // Skip closing tags `[/tag]`
    if inner.starts_with('/') {
        return None;
    }

    // Parse tag name (alphanumeric + _-)
    let tag_len = inner
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(inner.len());
    if tag_len == 0 {
        return None;
    }
    let tag = &inner[..tag_len];
    if !handlers.contains_key(tag) {
        return None;
    }

    // Find the closing `]` of the opening tag
    let after_tag = &inner[tag_len..];
    let bracket_close = after_tag.find(']')?;
    let attr_str = after_tag[..bracket_close].trim();
    let attrs = parse_attrs(attr_str);

    // Absolute position right after the opening `]`
    let after_open = start + 1 + tag_len + bracket_close + 1;

    // Look for a matching closing tag `[/tag]`
    let closing = format!("[/{}]", tag);
    let (inner_content, end) = match content[after_open..].find(&closing) {
        Some(rel_close) => {
            let text = content[after_open..after_open + rel_close].to_string();
            (Some(text), after_open + rel_close + closing.len())
        }
        None => (None, after_open),
    };

    let output = handlers[tag](&attrs, inner_content.as_deref());
    Some((output, end))
}

fn parse_attrs(s: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let mut remaining = s.trim();

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Read key up to '=' or whitespace
        let key_end = remaining
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(remaining.len());
        let key = remaining[..key_end].to_string();
        remaining = &remaining[key_end..];

        if remaining.starts_with('=') {
            remaining = &remaining[1..]; // skip '='
            if remaining.starts_with('"') {
                remaining = &remaining[1..]; // skip opening '"'
                let val_end = remaining.find('"').unwrap_or(remaining.len());
                attrs.insert(key, remaining[..val_end].to_string());
                remaining = &remaining[val_end..];
                if remaining.starts_with('"') {
                    remaining = &remaining[1..];
                }
            } else if remaining.starts_with('\'') {
                remaining = &remaining[1..];
                let val_end = remaining.find('\'').unwrap_or(remaining.len());
                attrs.insert(key, remaining[..val_end].to_string());
                remaining = &remaining[val_end..];
                if remaining.starts_with('\'') {
                    remaining = &remaining[1..];
                }
            } else {
                let val_end = remaining
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(remaining.len());
                attrs.insert(key, remaining[..val_end].to_string());
                remaining = &remaining[val_end..];
            }
        } else if !key.is_empty() {
            // Boolean flag attribute
            attrs.insert(key, "true".to_string());
        }
    }

    attrs
}

// ─── Built-in shortcodes ──────────────────────────────────────────────────────

fn register_builtins(r: &mut ShortcodeRegistry) {
    // [gallery ids="1,2,3" columns="3"]
    r.register("gallery", |attrs, _| {
        let ids = attrs.get("ids").map(|s| s.as_str()).unwrap_or("");
        let cols: u32 = attrs
            .get("columns")
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        let id_list: Vec<&str> = ids
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        if id_list.is_empty() {
            return String::new();
        }

        let mut html = format!(
            "<div class=\"wp-gallery\" style=\"display:grid;grid-template-columns:repeat({cols},1fr);gap:.5rem;margin:1rem 0;\">"
        );
        for id in &id_list {
            // Media served at /uploads/<id> placeholder; resolves if ID matches a numeric media ID
            let esc = html_attr_escape(id);
            html.push_str(&format!(
                "<figure class=\"gallery-item\"><img src=\"/api/media/{esc}\" loading=\"lazy\" \
                 style=\"width:100%;aspect-ratio:1;object-fit:cover;border-radius:4px;\"></figure>"
            ));
        }
        html.push_str("</div>");
        html
    });

    // [caption align="center" width="300"]inner content[/caption]
    r.register("caption", |attrs, content| {
        let align = attrs.get("align").map(|s| s.as_str()).unwrap_or("none");
        let width_style = attrs
            .get("width")
            .and_then(|s| s.parse::<u32>().ok())
            .map(|w| format!("max-width:{w}px;"))
            .unwrap_or_default();
        let text_align = match align {
            "aligncenter" | "center" => "center",
            "alignright" | "right" => "right",
            _ => "left",
        };
        let inner = content.unwrap_or("");
        format!(
            "<figure class=\"wp-caption\" \
             style=\"{width_style}text-align:{text_align};margin:1rem auto;\">\
             <figcaption class=\"wp-caption-text\">{inner}</figcaption></figure>"
        )
    });

    // [audio src="url.mp3"]
    r.register("audio", |attrs, _| {
        let src = match attrs.get("src") {
            Some(s) if !s.is_empty() => html_attr_escape(s),
            _ => return "<p><em>[audio: missing src attribute]</em></p>".to_string(),
        };
        format!(
            "<audio controls style=\"width:100%;margin:1rem 0;\">\
             <source src=\"{src}\">\
             <p>Your browser does not support audio playback.</p></audio>"
        )
    });
}

fn html_attr_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}