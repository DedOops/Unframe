use chrono::{DateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use tracing::warn;
use url::Url;

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub canonical_url: Option<Url>,
    pub scripts_discovered: u32,
    pub nodes: Vec<ParsedNode>,
}

#[derive(Debug, Clone)]
pub enum ParsedNode {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
    },
    Text {
        text: String,
    },
    List {
        ordered: bool,
        items: Vec<ParsedNode>,
    },
    ListItem {
        text: String,
        href: Option<String>,
        children: Vec<ParsedNode>,
    },
    Quote {
        text: String,
    },
    CodeBlock {
        code: String,
        language: Option<String>,
    },
    ImageRef {
        src: String,
        alt: Option<String>,
    },
    Link {
        text: String,
        href: String,
    },
    RichParagraph {
        segments: Vec<ParsedNode>,
    },
    Table {
        rows: Vec<TableRow>,
    },
    Divider,
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<Vec<ParsedNode>>,
    pub is_header: bool,
}

pub fn parse(html: &str, base_url: &Url) -> ParsedDocument {
    let html_str = decode_html(html);
    let document = Html::parse_document(&html_str);

    let mut scripts_discovered = 0u32;
    if let Ok(sel) = Selector::parse("script") {
        scripts_discovered = document.select(&sel).count() as u32;
    }

    let title = extract_meta_text(&document, "title");
    let og_title = extract_og(&document, "og:title");
    let og_description = extract_og(&document, "og:description");
    let description = extract_meta_name(&document, "description");
    let author = extract_meta_name(&document, "author");
    let language = extract_lang(&document);
    let canonical_url = extract_canonical(&document, base_url);
    let published_at = extract_published_at(&document);

    // Find content root: <main>, <article>, or body
    let content_root = find_content_root(&document);
    let nodes = if let Some(root) = content_root {
        extract_nodes(root, base_url)
    } else {
        vec![]
    };

    ParsedDocument {
        title,
        og_title,
        og_description,
        description,
        author,
        published_at,
        language,
        canonical_url,
        scripts_discovered,
        nodes,
    }
}

fn decode_html(html: &str) -> String {
    // encoding_rs handles BOM + charset; for now assume UTF-8 (bytes decoded upstream)
    html.to_string()
}

fn extract_meta_text(document: &Html, tag: &str) -> Option<String> {
    let sel = Selector::parse(tag).ok()?;
    let el = document.select(&sel).next()?;
    let text = el.text().collect::<String>().trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn extract_og(document: &Html, property: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[property=\"{property}\"]")).ok()?;
    let el = document.select(&sel).next()?;
    el.attr("content")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_meta_name(document: &Html, name: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[name=\"{name}\"]")).ok()?;
    let el = document.select(&sel).next()?;
    el.attr("content")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_lang(document: &Html) -> Option<String> {
    let sel = Selector::parse("html").ok()?;
    let el = document.select(&sel).next()?;
    el.attr("lang")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_canonical(document: &Html, base: &Url) -> Option<Url> {
    let sel = Selector::parse("link[rel=\"canonical\"]").ok()?;
    let el = document.select(&sel).next()?;
    let href = el.attr("href")?;
    base.join(href).ok()
}

fn extract_published_at(document: &Html) -> Option<DateTime<Utc>> {
    // Try <time datetime="...">, meta[property="article:published_time"]
    let sel = Selector::parse("time[datetime]").ok()?;
    let el = document.select(&sel).next()?;
    let dt_str = el.attr("datetime")?;
    dt_str.parse::<DateTime<Utc>>().ok()
}

fn find_content_root(document: &Html) -> Option<ElementRef<'_>> {
    // Priority: <main>, <article>, <body>
    for sel_str in &["main", "article", "body"] {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = document.select(&sel).next() {
                return Some(el);
            }
        }
    }
    None
}

const SKIP_TAGS: &[&str] = &[
    "nav", "footer", "header", "aside", "script", "style", "iframe", "noscript", "form",
];

fn should_skip(el: ElementRef) -> bool {
    let tag = el.value().name();
    SKIP_TAGS.contains(&tag)
}

fn resolve_url(href: &str, base: &Url) -> String {
    base.join(href)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| href.to_string())
}

fn extract_nodes(root: ElementRef, base_url: &Url) -> Vec<ParsedNode> {
    let mut nodes = Vec::new();
    collect_nodes(root, base_url, &mut nodes, 0);
    nodes
}

fn collect_nodes(el: ElementRef, base_url: &Url, out: &mut Vec<ParsedNode>, depth: usize) {
    if depth > 100 {
        warn!("DOM depth limit reached");
        return;
    }
    if should_skip(el) {
        return;
    }

    let tag = el.value().name();

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1) as u8;
            let text = clean_text(&el);
            if !text.is_empty() {
                out.push(ParsedNode::Heading { level, text });
            }
        }
        "p" => {
            let segments = collect_inline_segments(el, base_url);
            let has_links = segments
                .iter()
                .any(|s| matches!(s, ParsedNode::Link { .. }));
            if has_links {
                let non_empty: Vec<ParsedNode> = segments
                    .into_iter()
                    .filter(|s| match s {
                        ParsedNode::Text { text } | ParsedNode::Link { text, .. } => {
                            !text.is_empty()
                        }
                        _ => false,
                    })
                    .collect();
                if !non_empty.is_empty() {
                    out.push(ParsedNode::RichParagraph {
                        segments: non_empty,
                    });
                }
            } else {
                let text = clean_text(&el);
                if !text.is_empty() {
                    out.push(ParsedNode::Paragraph { text });
                }
            }
        }
        "ul" | "ol" => {
            let ordered = tag == "ol";
            let items = collect_list_items(el, base_url, depth + 1);
            if !items.is_empty() {
                out.push(ParsedNode::List { ordered, items });
            }
        }
        "blockquote" => {
            let text = clean_text(&el);
            if !text.is_empty() {
                out.push(ParsedNode::Quote { text });
            }
        }
        "pre" => {
            let code = collect_code(el);
            let language = detect_code_language(el);
            if !code.is_empty() {
                out.push(ParsedNode::CodeBlock { code, language });
            }
        }
        "code"
            if el
                .parent()
                .and_then(|p| p.value().as_element())
                .map(|e| e.name() != "pre")
                .unwrap_or(true) =>
        {
            // inline code — skip as separate node, handled by parent text
        }
        "img" => {
            let src = el.attr("src").unwrap_or("").to_string();
            if !src.is_empty() {
                let src = resolve_url(&src, base_url);
                let alt = el
                    .attr("alt")
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());
                out.push(ParsedNode::ImageRef { src, alt });
            }
        }
        "a" => {
            let text = clean_text(&el);
            let href = el.attr("href").unwrap_or("").to_string();
            if !text.is_empty() && !href.is_empty() {
                let href = resolve_url(&href, base_url);
                out.push(ParsedNode::Link { text, href });
            }
        }
        "table" => {
            let rows = collect_table_rows(el, base_url);
            if !rows.is_empty() {
                out.push(ParsedNode::Table { rows });
            }
        }
        "hr" => {
            out.push(ParsedNode::Divider);
        }
        _ => {
            // Recurse into div, section, article, main, span, etc.
            for child in el.children() {
                if let Some(child_el) = ElementRef::wrap(child) {
                    collect_nodes(child_el, base_url, out, depth + 1);
                }
            }
        }
    }
}

fn collect_inline_segments(el: ElementRef, base_url: &Url) -> Vec<ParsedNode> {
    let mut segments: Vec<ParsedNode> = Vec::new();
    for child in el.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            let tag = child_el.value().name();
            if tag == "a" {
                let text = clean_text(&child_el);
                if !text.is_empty() {
                    if let Some(href) = child_el.attr("href").filter(|h| !h.is_empty()) {
                        let href = resolve_url(href, base_url);
                        segments.push(ParsedNode::Link { text, href });
                    } else {
                        append_text(&mut segments, text);
                    }
                }
            } else {
                // inline elements: em, strong, span, code, etc.
                let text = clean_text(&child_el);
                if !text.is_empty() {
                    append_text(&mut segments, text);
                }
            }
        } else if let scraper::node::Node::Text(t) = child.value() {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                let has_leading = t.starts_with(|c: char| c.is_ascii_whitespace());
                let has_trailing = t.ends_with(|c: char| c.is_ascii_whitespace());
                let core = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
                let mut normalized = String::new();
                if has_leading {
                    normalized.push(' ');
                }
                normalized.push_str(&core);
                if has_trailing {
                    normalized.push(' ');
                }
                append_text(&mut segments, normalized);
            }
        }
    }
    segments
}

fn append_text(segments: &mut Vec<ParsedNode>, text: String) {
    if let Some(ParsedNode::Text { text: prev }) = segments.last_mut() {
        prev.push(' ');
        prev.push_str(&text);
    } else {
        segments.push(ParsedNode::Text { text });
    }
}

fn clean_text(el: &ElementRef) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_list_items(el: ElementRef, base_url: &Url, depth: usize) -> Vec<ParsedNode> {
    if depth > 100 {
        return vec![];
    }
    let mut items = Vec::new();
    for child in el.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            if child_el.value().name() == "li" {
                let text = clean_text(&child_el);
                if !text.is_empty() {
                    let href = extract_primary_link_href(child_el, base_url);
                    let mut children = Vec::new();
                    for sub in child_el.children() {
                        if let Some(sub_el) = ElementRef::wrap(sub) {
                            let sub_tag = sub_el.value().name();
                            if sub_tag == "ul" || sub_tag == "ol" {
                                let ordered = sub_tag == "ol";
                                let sub_items = collect_list_items(sub_el, base_url, depth + 1);
                                if !sub_items.is_empty() {
                                    children.push(ParsedNode::List {
                                        ordered,
                                        items: sub_items,
                                    });
                                }
                            }
                        }
                    }
                    items.push(ParsedNode::ListItem {
                        text,
                        href,
                        children,
                    });
                }
            }
        }
    }
    items
}

fn extract_primary_link_href(li: ElementRef, base_url: &Url) -> Option<String> {
    for child in li.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            if child_el.value().name() == "a" {
                if let Some(href) = child_el.attr("href") {
                    if !href.is_empty() {
                        return Some(resolve_url(href, base_url));
                    }
                }
            }
        }
    }
    None
}

fn collect_code(el: ElementRef) -> String {
    // Find <code> inside <pre>, or use text directly
    if let Ok(code_sel) = Selector::parse("code") {
        if let Some(code_el) = el.select(&code_sel).next() {
            return code_el.text().collect::<String>();
        }
    }
    el.text().collect::<String>()
}

fn detect_code_language(el: ElementRef) -> Option<String> {
    // <pre><code class="language-rust"> or <pre class="rust">
    if let Ok(code_sel) = Selector::parse("code") {
        if let Some(code_el) = el.select(&code_sel).next() {
            let class = code_el.attr("class").unwrap_or("");
            for part in class.split_whitespace() {
                if let Some(lang) = part.strip_prefix("language-") {
                    return Some(lang.to_string());
                }
            }
        }
    }
    None
}

fn collect_table_rows(el: ElementRef, base_url: &Url) -> Vec<TableRow> {
    let mut rows = Vec::new();
    if let Ok(tr_sel) = Selector::parse("tr") {
        for tr in el.select(&tr_sel) {
            let mut cells: Vec<Vec<ParsedNode>> = Vec::new();
            let mut is_header = false;
            for child in tr.children() {
                if let Some(cell) = ElementRef::wrap(child) {
                    let tag = cell.value().name();
                    if tag == "th" {
                        is_header = true;
                        let mut cell_nodes = Vec::new();
                        collect_nodes(cell, base_url, &mut cell_nodes, 0);
                        // Fallback: if no semantic nodes, use raw text as paragraph
                        if cell_nodes.is_empty() {
                            let text = clean_text(&cell);
                            if !text.is_empty() {
                                cell_nodes.push(ParsedNode::Text { text });
                            }
                        }
                        cells.push(cell_nodes);
                    } else if tag == "td" {
                        let mut cell_nodes = Vec::new();
                        collect_nodes(cell, base_url, &mut cell_nodes, 0);
                        if cell_nodes.is_empty() {
                            let text = clean_text(&cell);
                            if !text.is_empty() {
                                cell_nodes.push(ParsedNode::Text { text });
                            }
                        }
                        cells.push(cell_nodes);
                    }
                }
            }
            if !cells.is_empty() {
                rows.push(TableRow { cells, is_header });
            }
        }
    }
    rows
}

/// Detect charset from HTTP Content-Type header and raw response bytes.
///
/// Returns `(html_string, charset_name, source)` where source is one of:
/// `"http-header"`, `"bom"`, `"meta-charset"`, `"meta-http-equiv"`, `"default"`.
pub fn detect_charset(content_type_raw: &str, body: &[u8]) -> (String, String, String) {
    // 1. HTTP Content-Type header: charset=...
    if let Some(cs) = charset_from_content_type(content_type_raw) {
        if let Some(enc) = encoding_rs::Encoding::for_label(cs.as_bytes()) {
            let (html, _, _) = enc.decode(body);
            return (
                html.into_owned(),
                enc.name().to_ascii_lowercase(),
                "http-header".into(),
            );
        }
    }

    // 2. BOM
    if body.starts_with(b"\xEF\xBB\xBF") {
        return (
            String::from_utf8_lossy(&body[3..]).into_owned(),
            "utf-8".into(),
            "bom".into(),
        );
    }
    if body.starts_with(b"\xFF\xFE") {
        let (html, _, _) = encoding_rs::UTF_16LE.decode(body);
        return (html.into_owned(), "utf-16le".into(), "bom".into());
    }
    if body.starts_with(b"\xFE\xFF") {
        let (html, _, _) = encoding_rs::UTF_16BE.decode(body);
        return (html.into_owned(), "utf-16be".into(), "bom".into());
    }

    // 3 & 4. Scan first 1 KB (as lossy UTF-8) for <meta charset> / http-equiv
    let sniff_len = body.len().min(1024);
    let preview = String::from_utf8_lossy(&body[..sniff_len]).to_ascii_lowercase();

    if let Some((cs, source)) = charset_from_meta(&preview) {
        if let Some(enc) = encoding_rs::Encoding::for_label(cs.as_bytes()) {
            let (html, _, _) = enc.decode(body);
            return (html.into_owned(), enc.name().to_ascii_lowercase(), source);
        }
    }

    // 5. Default: UTF-8 (lossy)
    (
        String::from_utf8_lossy(body).into_owned(),
        "utf-8".into(),
        "default".into(),
    )
}

fn charset_from_content_type(ct: &str) -> Option<String> {
    let lower = ct.to_ascii_lowercase();
    let start = lower.find("charset=")? + 8;
    let rest = ct[start..].trim_start_matches(['"', '\'']);
    let end = rest
        .find(|c: char| c == '"' || c == '\'' || c == ';' || c.is_ascii_whitespace())
        .unwrap_or(rest.len());
    let cs = rest[..end].trim();
    if cs.is_empty() {
        None
    } else {
        Some(cs.to_string())
    }
}

fn charset_from_meta(preview_lower: &str) -> Option<(String, String)> {
    // <meta charset="windows-1251">
    if let Some(idx) = preview_lower.find("<meta charset=") {
        let after = preview_lower[idx + 14..].trim_start_matches(['"', '\'']);
        let end = after
            .find(|c: char| c == '"' || c == '\'' || c == '>' || c.is_ascii_whitespace())
            .unwrap_or(after.len());
        let cs = after[..end].trim();
        if !cs.is_empty() {
            return Some((cs.to_string(), "meta-charset".into()));
        }
    }

    // <meta http-equiv="content-type" content="text/html; charset=...">
    if let Some(idx) = preview_lower.find("http-equiv") {
        let area_end = (idx + 256).min(preview_lower.len());
        let area = &preview_lower[idx..area_end];
        if area.contains("content-type") {
            if let Some(cs) = charset_from_content_type(area) {
                return Some((cs, "meta-http-equiv".into()));
            }
        }
    }

    None
}

#[cfg(test)]
mod charset_tests {
    use super::*;

    #[test]
    fn utf8_default() {
        let (html, cs, src) = detect_charset("text/html", b"<p>hello</p>");
        assert_eq!(cs, "utf-8");
        assert_eq!(src, "default");
        assert!(html.contains("hello"));
    }

    #[test]
    fn charset_from_http_header() {
        let (_, cs, src) = detect_charset(
            "text/html; charset=windows-1251",
            b"\xCF\xF0\xE8\xE2\xE5\xF2",
        );
        assert_eq!(cs, "windows-1251");
        assert_eq!(src, "http-header");
    }

    #[test]
    fn charset_from_utf8_bom() {
        let body = b"\xEF\xBB\xBFhello";
        let (html, cs, src) = detect_charset("text/html", body);
        assert_eq!(cs, "utf-8");
        assert_eq!(src, "bom");
        assert_eq!(html, "hello");
    }

    #[test]
    fn charset_from_meta_charset() {
        let body = b"<html><head><meta charset=\"windows-1251\"></head></html>";
        let (_, cs, src) = detect_charset("text/html", body);
        assert_eq!(cs, "windows-1251");
        assert_eq!(src, "meta-charset");
    }

    #[test]
    fn charset_from_iso_8859_1() {
        // encoding_rs follows WHATWG: iso-8859-1 is an alias for windows-1252
        let (_, cs, src) = detect_charset("text/html; charset=iso-8859-1", b"<p>caf\xE9</p>");
        assert_eq!(cs, "windows-1252");
        assert_eq!(src, "http-header");
    }
}
