/// Askama custom filters — usable in templates as `{{ value | filter_name }}`.
///
/// Askama 0.14 injects `__askama_values: &dyn Values` as the second argument
/// for every custom filter, so all functions must accept it (conventionally `_v`).
///
/// Usage examples:
///   {{ post.created_at | rl_date("%Y-%m-%d") }}
///   {{ post.content    | rl_excerpt(150) }}
///   {{ post.content    | strip_md }}

use askama::Values;

/// Format a `chrono::DateTime<Utc>` using a strftime format string.
///
/// Template: `{{ post.created_at | rl_date("%B %d, %Y") }}`
pub fn rl_date(
    value: &chrono::DateTime<chrono::Utc>,
    _v: &dyn Values,
    fmt: &str,
) -> askama::Result<String> {
    Ok(value.format(fmt).to_string())
}

/// Generate a plain-text excerpt by stripping basic Markdown and truncating.
/// Returns the content unchanged if it is already shorter than `max_chars`.
///
/// Template: `{{ post.content | rl_excerpt(150) }}`
pub fn rl_excerpt(content: &str, _v: &dyn Values, max_chars: usize) -> askama::Result<String> {
    let plain = strip_markdown(content);
    if plain.chars().count() <= max_chars {
        return Ok(plain);
    }
    let truncated: String = plain.chars().take(max_chars).collect();
    let cut = truncated.rfind(' ').unwrap_or(truncated.len());
    Ok(format!("{}…", &truncated[..cut]))
}

/// Strip basic Markdown syntax from a string, returning plain text.
/// Useful for generating meta descriptions or feed summaries.
///
/// Template: `{{ post.content | strip_md }}`
pub fn strip_md(content: &str, _v: &dyn Values) -> askama::Result<String> {
    Ok(strip_markdown(content))
}

pub(crate) fn strip_markdown(s: &str) -> String {
    let mut out = String::new();
    let mut in_code_block = false;

    for line in s.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        // Strip ATX headings
        let line = trimmed.trim_start_matches('#').trim();
        // Strip blockquote markers
        let line = line.trim_start_matches('>').trim();
        // Strip list markers
        let line = if line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with("+ ")
        {
            &line[2..]
        } else {
            line
        };

        let line = strip_inline(line);
        if !line.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&line);
        }
    }

    out
}

/// Remove inline Markdown: links `[text](url)` → `text`, emphasis `*`/`_`, inline code `` ` ``.
fn strip_inline(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            // Collect link text
            let mut inner = String::new();
            for c2 in chars.by_ref() {
                if c2 == ']' {
                    break;
                }
                inner.push(c2);
            }
            // Skip (url) if present
            if chars.peek() == Some(&'(') {
                chars.next();
                for c2 in chars.by_ref() {
                    if c2 == ')' {
                        break;
                    }
                }
            }
            result.push_str(&inner);
        } else if c == '`' || c == '*' || c == '_' {
            continue;
        } else {
            result.push(c);
        }
    }
    result
}