use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

struct Style {
    src: String,
}

struct Script {
    src: String,
    in_footer: bool,
}

pub struct AssetRegistry {
    styles: HashMap<String, Style>,
    scripts: HashMap<String, Script>,
    style_order: Vec<String>,
    script_order: Vec<String>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
            scripts: HashMap::new(),
            style_order: Vec::new(),
            script_order: Vec::new(),
        }
    }

    /// Register a stylesheet. Duplicate handles are ignored.
    pub fn enqueue_style(&mut self, handle: &str, src: &str) {
        if !self.styles.contains_key(handle) {
            self.style_order.push(handle.to_string());
            self.styles.insert(handle.to_string(), Style { src: src.to_string() });
        }
    }

    /// Register a script. `in_footer = true` defers it to `rl_footer`.
    pub fn enqueue_script(&mut self, handle: &str, src: &str, in_footer: bool) {
        if !self.scripts.contains_key(handle) {
            self.script_order.push(handle.to_string());
            self.scripts.insert(handle.to_string(), Script { src: src.to_string(), in_footer });
        }
    }

    /// Render `<link>` and head `<script>` tags.
    pub fn render_head(&self) -> String {
        let mut out = String::new();
        for h in &self.style_order {
            if let Some(s) = self.styles.get(h) {
                out.push_str(&format!(
                    "<link rel=\"stylesheet\" href=\"{}\">\n",
                    escape_attr(&s.src)
                ));
            }
        }
        for h in &self.script_order {
            if let Some(s) = self.scripts.get(h) {
                if !s.in_footer {
                    out.push_str(&format!(
                        "<script src=\"{}\"></script>\n",
                        escape_attr(&s.src)
                    ));
                }
            }
        }
        out
    }

    /// Render footer `<script>` tags.
    pub fn render_footer(&self) -> String {
        let mut out = String::new();
        for h in &self.script_order {
            if let Some(s) = self.scripts.get(h) {
                if s.in_footer {
                    out.push_str(&format!(
                        "<script src=\"{}\"></script>\n",
                        escape_attr(&s.src)
                    ));
                }
            }
        }
        out
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

static INSTANCE: OnceLock<Mutex<AssetRegistry>> = OnceLock::new();

fn registry() -> &'static Mutex<AssetRegistry> {
    INSTANCE.get_or_init(|| Mutex::new(AssetRegistry::new()))
}

/// Enqueue a stylesheet globally (once, by handle).
pub fn enqueue_style(handle: &str, src: &str) {
    registry().lock().unwrap().enqueue_style(handle, src);
}

/// Enqueue a script globally. Pass `in_footer = true` to defer to rl_footer.
pub fn enqueue_script(handle: &str, src: &str, in_footer: bool) {
    registry().lock().unwrap().enqueue_script(handle, src, in_footer);
}

/// Render all enqueued head assets as an HTML string.
pub fn render_head() -> String {
    registry().lock().unwrap().render_head()
}

/// Render all enqueued footer assets as an HTML string.
pub fn render_footer() -> String {
    registry().lock().unwrap().render_footer()
}