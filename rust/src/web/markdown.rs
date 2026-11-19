//! A small, dependency-free Markdown renderer for the in-repo documentation.
//!
//! The documentation only uses a deliberately narrow subset of Markdown:
//! ATX headings, paragraphs, fenced code blocks, nested lists, tables, bold
//! text, inline code and links (plus the occasional block quote / rule).
//! Supporting exactly that keeps the site free of a Markdown dependency and
//! keeps the renderer easy to audit. Raw HTML and images are never emitted.

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RenderedDoc {
    pub html: String,
    pub headings: Vec<Heading>,
}

/// Renders a Markdown document to HTML and returns the headings (for the
/// "on this page" table of contents).
pub fn render(markdown: &str) -> RenderedDoc {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    let mut html = String::new();
    let mut headings = Vec::new();
    let mut used_ids: HashSet<String> = HashSet::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        if let Some((fence_len, lang)) = fence_start(line) {
            let (code, next) = collect_fence(&lines, i, fence_len);
            let language = if lang.is_empty() {
                String::new()
            } else {
                format!(" class=\"language-{}\"", escape_html(&lang))
            };
            html.push_str("<pre class=\"code-block\"><code");
            html.push_str(&language);
            html.push('>');
            html.push_str(&escape_html(&code));
            html.push_str("</code></pre>\n");
            i = next;
            continue;
        }

        if is_table_start(&lines, i) {
            let (table, next) = render_table(&lines, i);
            html.push_str(&table);
            i = next;
            continue;
        }

        if let Some((level, text)) = heading(line) {
            let plain = plain_heading_text(text);
            let id = unique_id(&plain, &mut used_ids);
            headings.push(Heading {
                level,
                id: id.clone(),
                text: plain,
            });
            html.push_str(&format!(
                "<h{level} id=\"{id}\">{text}<a class=\"heading-anchor\" href=\"#{id}\" aria-label=\"Permalink\">#</a></h{level}>\n",
                level = level,
                id = id,
                text = render_inline(text)
            ));
            i += 1;
            continue;
        }

        if list_marker(line).is_some() {
            let (list, next) = render_list(&lines, i);
            html.push_str(&list);
            i = next;
            continue;
        }

        if line.trim() == "---" || line.trim() == "***" {
            html.push_str("<hr>\n");
            i += 1;
            continue;
        }

        if let Some(quote) = line.trim_start().strip_prefix("> ") {
            let mut parts = vec![quote.trim()];
            i += 1;
            while i < lines.len() {
                let next = lines[i];
                if let Some(inner) = next.trim_start().strip_prefix("> ") {
                    parts.push(inner.trim());
                    i += 1;
                } else if next.trim().is_empty() {
                    i += 1;
                    break;
                } else {
                    break;
                }
            }
            html.push_str("<blockquote><p>");
            html.push_str(&render_inline(&parts.join(" ")));
            html.push_str("</p></blockquote>\n");
            continue;
        }

        // Paragraph: consume until a blank line or the start of another block.
        let mut parts = vec![line.trim()];
        i += 1;
        while i < lines.len() && !lines[i].trim().is_empty() && !starts_block(&lines, i) {
            parts.push(lines[i].trim());
            i += 1;
        }
        html.push_str("<p>");
        html.push_str(&render_inline(&parts.join(" ")));
        html.push_str("</p>\n");
    }

    RenderedDoc { html, headings }
}

fn starts_block(lines: &[&str], i: usize) -> bool {
    let line = lines[i];
    if heading(line).is_some()
        || fence_start(line).is_some()
        || list_marker(line).is_some()
        || line.trim() == "---"
        || line.trim() == "***"
        || line.trim_start().starts_with("> ")
        || is_table_start(lines, i)
    {
        return true;
    }
    false
}

fn heading(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    let text = rest.strip_prefix(' ')?.trim();
    if text.is_empty() {
        return None;
    }
    Some((hashes as u8, text))
}

fn fence_start(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let ticks = trimmed.chars().take_while(|c| *c == '`').count();
    if ticks < 3 {
        return None;
    }
    let info = trimmed[ticks..].trim();
    let lang = info
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(',');
    Some((ticks, lang.to_string()))
}

fn collect_fence(lines: &[&str], start: usize, fence_len: usize) -> (String, usize) {
    let mut code = String::new();
    let mut i = start + 1;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let ticks = trimmed.chars().take_while(|c| *c == '`').count();
        if ticks >= fence_len && trimmed[ticks..].trim().is_empty() {
            return (code.trim_end_matches('\n').to_string(), i + 1);
        }
        code.push_str(lines[i]);
        code.push('\n');
        i += 1;
    }
    (code.trim_end_matches('\n').to_string(), i)
}

fn is_table_start(lines: &[&str], i: usize) -> bool {
    if i + 1 >= lines.len() {
        return false;
    }
    let header = lines[i].trim();
    let separator = lines[i + 1].trim();
    if !header.starts_with('|') || !separator.starts_with('|') {
        return false;
    }
    let cells: Vec<&str> = split_row(separator);
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let cell = cell.trim_matches(':').trim();
            cell.len() >= 3 && cell.chars().all(|c| c == '-')
        })
}

fn split_row(line: &str) -> Vec<&str> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

fn render_table(lines: &[&str], start: usize) -> (String, usize) {
    let header = split_row(lines[start]);
    let mut html = String::from("<div class=\"table-wrap\"><table>\n<thead><tr>");
    for cell in &header {
        html.push_str("<th>");
        html.push_str(&render_inline(cell));
        html.push_str("</th>");
    }
    html.push_str("</tr></thead>\n<tbody>\n");

    let mut i = start + 2;
    while i < lines.len() {
        let line = lines[i].trim();
        if !line.starts_with('|') {
            break;
        }
        let cells = split_row(line);
        html.push_str("<tr>");
        for index in 0..header.len() {
            html.push_str("<td>");
            if let Some(cell) = cells.get(index) {
                html.push_str(&render_inline(cell));
            }
            html.push_str("</td>");
        }
        html.push_str("</tr>\n");
        i += 1;
    }
    html.push_str("</tbody></table></div>\n");
    (html, i)
}

/// `(indent, ordered, text)` for a list item line.
fn list_marker(line: &str) -> Option<(usize, bool, &str)> {
    let mut indent = 0;
    let mut rest = line;
    for ch in line.chars() {
        match ch {
            ' ' => indent += 1,
            '\t' => indent += 4,
            _ => break,
        }
        rest = &rest[ch.len_utf8()..];
    }
    if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
        return Some((indent, false, rest[2..].trim()));
    }
    let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        let after = &rest[digits..];
        if let Some(text) = after.strip_prefix(". ") {
            return Some((indent, true, text.trim()));
        }
    }
    None
}

fn render_list(lines: &[&str], start: usize) -> (String, usize) {
    let (indent, ordered, _) = list_marker(lines[start]).expect("list item");
    let mut html = String::new();
    let mut i = start;
    render_list_level(lines, &mut i, indent, ordered, &mut html);
    (html, i)
}

fn render_list_level(
    lines: &[&str],
    i: &mut usize,
    indent: usize,
    ordered: bool,
    html: &mut String,
) {
    html.push_str(if ordered { "<ol>\n" } else { "<ul>\n" });
    while *i < lines.len() {
        let Some((item_indent, item_ordered, text)) = list_marker(lines[*i]) else {
            break;
        };
        if item_indent < indent {
            break;
        }
        if item_indent > indent {
            // A nested list is consumed by the parent item below.
            break;
        }
        if item_ordered != ordered {
            break;
        }

        html.push_str("<li>");
        html.push_str(&render_inline(text));
        *i += 1;

        if *i < lines.len() {
            if let Some((next_indent, next_ordered, _)) = list_marker(lines[*i]) {
                if next_indent > indent {
                    html.push('\n');
                    render_list_level(lines, i, next_indent, next_ordered, html);
                }
            }
        }
        html.push_str("</li>\n");
    }
    html.push_str(if ordered { "</ol>\n" } else { "</ul>\n" });
}

fn unique_id(text: &str, used: &mut HashSet<String>) -> String {
    let base = slugify(text);
    let mut candidate = base.clone();
    let mut n = 2;
    while used.contains(&candidate) {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    used.insert(candidate.clone());
    candidate
}

fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut hyphen = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            hyphen = false;
        } else if !hyphen && !out.is_empty() {
            out.push('-');
            hyphen = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

fn plain_heading_text(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '`' | '*' => {}
            '[' => {
                // Keep the link label, drop the target.
                let mut label = String::new();
                let mut depth = 0;
                for inner in chars.by_ref() {
                    match inner {
                        '[' => depth += 1,
                        ']' if depth == 0 => break,
                        ']' => {
                            depth -= 1;
                            label.push(inner);
                        }
                        _ => label.push(inner),
                    }
                }
                out.push_str(&label);
                // Skip the "(url)" part.
                if chars.peek() == Some(&'(') {
                    for inner in chars.by_ref() {
                        if inner == ')' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(ch),
        }
    }
    out.trim().to_string()
}

/// Escapes text for use in HTML text or attribute values.
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn render_inline(text: &str) -> String {
    let without_images = strip_images(text);
    let mut out = String::new();
    let mut segment = String::new();
    let mut in_code = false;

    for ch in without_images.chars() {
        if ch == '`' {
            if in_code {
                out.push_str("<code>");
                out.push_str(&escape_html(&segment));
                out.push_str("</code>");
            } else {
                out.push_str(&render_emphasis(&escape_html(&segment)));
            }
            segment.clear();
            in_code = !in_code;
        } else {
            segment.push(ch);
        }
    }

    if in_code {
        // Unbalanced backtick: render the remainder as plain text.
        out.push('`');
        out.push_str(&render_emphasis(&escape_html(&segment)));
    } else {
        out.push_str(&render_emphasis(&escape_html(&segment)));
    }
    out
}

/// Removes `![alt](url)` image syntax; the site never loads external images.
fn strip_images(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            if let Some(end) = image_end(&chars, i) {
                i = end;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn image_end(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start + 2;
    let mut depth = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => depth += 1,
            ']' if depth == 0 => {
                if chars.get(i + 1) == Some(&'(') {
                    let mut j = i + 2;
                    while j < chars.len() {
                        if chars[j] == ')' {
                            return Some(j + 1);
                        }
                        j += 1;
                    }
                }
                return None;
            }
            ']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

fn render_emphasis(escaped: &str) -> String {
    let bolded = render_bold(escaped);
    render_links(&bolded)
}

fn render_bold(escaped: &str) -> String {
    let mut out = String::new();
    let mut rest = escaped;
    while let Some(open) = rest.find("**") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        if let Some(close) = after.find("**") {
            out.push_str("<strong>");
            out.push_str(&after[..close]);
            out.push_str("</strong>");
            rest = &after[close + 2..];
        } else {
            out.push_str("**");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn render_links(escaped: &str) -> String {
    let mut out = String::new();
    let mut rest = escaped;
    while let Some(open) = rest.find('[') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find("](") else {
            out.push('[');
            rest = after;
            continue;
        };
        let label = &after[..close];
        let after_label = &after[close + 2..];
        let Some(end) = after_label.find(')') else {
            out.push('[');
            rest = after;
            continue;
        };
        let url = after_label[..end].trim();
        if is_safe_url(url) {
            out.push_str("<a href=\"");
            out.push_str(url);
            out.push_str("\">");
            out.push_str(label);
            out.push_str("</a>");
            rest = &after_label[end + 1..];
        } else {
            out.push('[');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn is_safe_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with('/')
        || url.starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_headings_and_collects_toc() {
        let doc = render("# Title\n\n## Get Weapons\n\n### By Tier\n");
        assert!(doc.html.contains("<h1 id=\"title\">"));
        assert!(doc.html.contains("<h2 id=\"get-weapons\">"));
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[1].id, "get-weapons");
        assert_eq!(doc.headings[2].level, 3);
    }

    #[test]
    fn escapes_html_and_javascript_links() {
        let doc = render("Use <script>alert(1)</script> **bold** [x](javascript:alert(1))");
        assert!(!doc.html.contains("<script>"));
        assert!(doc.html.contains("&lt;script&gt;"));
        assert!(doc.html.contains("<strong>bold</strong>"));
        assert!(!doc.html.contains("href=\"javascript:"));
    }

    #[test]
    fn renders_code_spans_and_fences() {
        let doc = render("Run `cargo test` now.\n\n```json\n{\"data\": [1, 2]}\n```\n");
        assert!(doc.html.contains("<code>cargo test</code>"));
        assert!(doc.html.contains("class=\"language-json\""));
        assert!(doc.html.contains("&quot;data&quot;"));
        assert!(!doc.html.contains("<pre><code>{\"data\""));
    }

    #[test]
    fn renders_nested_lists() {
        let doc = render(
            "- Method: GET\n- Query Parameters:\n  - `tier`: filter\n  - `category_id`: filter\n",
        );
        assert!(doc.html.contains("<ul>"));
        assert!(doc.html.contains("<li>Query Parameters:"));
        assert_eq!(doc.html.matches("<ul>").count(), 2);
        assert!(doc.html.contains("<code>tier</code>"));
    }

    #[test]
    fn renders_tables() {
        let doc = render("| Key | Description |\n| :--- | :--- |\n| weapon | Weapon |\n");
        assert!(doc.html.contains("<table>"));
        assert!(doc.html.contains("<th>Key</th>"));
        assert!(doc.html.contains("<td>Weapon</td>"));
    }

    #[test]
    fn drops_images() {
        let doc = render("![Preview](https://res.cloudinary.com/example.png)\n\nHello");
        assert!(!doc.html.contains("<img"));
        assert!(!doc.html.contains("cloudinary"));
        assert!(doc.html.contains("Hello"));
    }

    #[test]
    fn renders_relative_links() {
        let doc = render("See [Versions](/docs/versions).");
        assert!(doc.html.contains("<a href=\"/docs/versions\">Versions</a>"));
    }

    #[test]
    fn heading_ids_are_deduped() {
        let doc = render("## Same\n\n## Same\n");
        assert!(doc.html.contains("id=\"same\""));
        assert!(doc.html.contains("id=\"same-2\""));
    }
}
