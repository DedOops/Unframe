use chrono::Utc;
use thiserror::Error;
use tracing::info;
use url::Url;
use uuid::Uuid;

use unframe_model::{
    Block, DocumentMetadata, InformationDocument, InlineContent, NetworkAudit, OutlineItem,
    SourceInfo, MODEL_VERSION,
};
use unframe_network::FetchResult;
use unframe_parser::{ParsedDocument, ParsedNode, TableRow};

pub const ADAPTER_NAME: &str = "GenericHtmlAdapter";
pub const ADAPTER_VERSION: &str = "0.1.0";

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub enum AdapterResult {
    Success(InformationDocument),
    PartialSuccess(InformationDocument),
    Unsupported(String),
    Failed(AdapterError),
}

pub fn adapt(parsed: ParsedDocument, fetch: &FetchResult) -> AdapterResult {
    info!(
        "adapting document from {} ({} nodes)",
        fetch.final_url,
        parsed.nodes.len()
    );

    // Count total text chars to detect empty SPA
    let total_text: usize = count_text_chars(&parsed.nodes);
    if total_text < 100 && parsed.nodes.is_empty() {
        return AdapterResult::Unsupported(
            "Page appears to require JavaScript to render content".into(),
        );
    }

    let mut warnings = Vec::new();
    let title = parsed
        .og_title
        .clone()
        .or_else(|| parsed.title.clone())
        .unwrap_or_else(|| {
            warnings.push("No title found".into());
            fetch.final_url.to_string()
        });

    let mut blocks = Vec::new();
    let mut outline = Vec::new();

    for node in &parsed.nodes {
        convert_node(node, &mut blocks, &mut outline);
    }

    if blocks.is_empty() {
        return AdapterResult::Unsupported("No readable content could be extracted".into());
    }

    let mut audit = fetch.audit.clone();
    audit.scripts_discovered = parsed.scripts_discovered;
    audit.adapter_name = ADAPTER_NAME.into();
    audit.adapter_version = ADAPTER_VERSION.into();

    let doc = InformationDocument {
        id: Uuid::new_v4().to_string(),
        model_version: MODEL_VERSION.into(),
        source: SourceInfo {
            requested_url: fetch.requested_url.to_string(),
            final_url: fetch.final_url.to_string(),
            retrieved_at: fetch.retrieved_at,
            content_type: fetch.content_type.clone(),
            charset: "utf-8".into(),
            charset_source: "default".into(),
        },
        metadata: DocumentMetadata {
            title,
            description: parsed.og_description.or(parsed.description),
            author: parsed.author,
            published_at: parsed.published_at,
            language: parsed.language,
            canonical_url: parsed.canonical_url.map(|u| u.to_string()),
        },
        blocks,
        outline,
        audit,
        warnings,
    };

    if doc.warnings.is_empty() {
        AdapterResult::Success(doc)
    } else {
        AdapterResult::PartialSuccess(doc)
    }
}

/// Adapt from already-parsed HTML bytes (convenience for offline/fixture use)
pub fn adapt_html(html: &str, url: &Url) -> AdapterResult {
    let parsed = unframe_parser::parse(html, url);
    let fetch = FetchResult {
        requested_url: url.clone(),
        final_url: url.clone(),
        content_type: "text/html".into(),
        content_type_raw: "text/html".into(),
        body: bytes::Bytes::new(),
        audit: NetworkAudit::default(),
        retrieved_at: Utc::now(),
    };
    adapt(parsed, &fetch)
}

fn count_text_chars(nodes: &[ParsedNode]) -> usize {
    nodes
        .iter()
        .map(|n| match n {
            ParsedNode::Paragraph { text } | ParsedNode::Text { text } => text.len(),
            ParsedNode::Heading { text, .. } => text.len(),
            ParsedNode::List { items, .. } => count_text_chars(items),
            ParsedNode::ListItem { text, children, .. } => text.len() + count_text_chars(children),
            ParsedNode::RichParagraph { segments } => segments
                .iter()
                .map(|s| match s {
                    ParsedNode::Text { text } | ParsedNode::Link { text, .. } => text.len(),
                    _ => 0,
                })
                .sum(),
            ParsedNode::Quote { text } => text.len(),
            ParsedNode::CodeBlock { code, .. } => code.len(),
            _ => 0,
        })
        .sum()
}

fn convert_node(node: &ParsedNode, blocks: &mut Vec<Block>, outline: &mut Vec<OutlineItem>) {
    match node {
        ParsedNode::Heading { level, text } => {
            if *level <= 3 {
                outline.push(OutlineItem {
                    level: *level,
                    text: text.clone(),
                    block_index: blocks.len(),
                });
            }
            blocks.push(Block::Heading {
                level: *level,
                text: text.clone(),
                id: Some(slugify(text)),
            });
        }
        ParsedNode::Paragraph { text } => {
            blocks.push(Block::Paragraph { text: text.clone() });
        }
        ParsedNode::Text { text } => {
            blocks.push(Block::Text { text: text.clone() });
        }
        ParsedNode::List { ordered, items } => {
            let converted: Vec<Block> = items.iter().map(convert_node_to_block).collect();
            blocks.push(Block::List {
                ordered: *ordered,
                items: converted,
            });
        }
        ParsedNode::RichParagraph { segments } => {
            let inline: Vec<InlineContent> = segments
                .iter()
                .filter_map(|s| match s {
                    ParsedNode::Text { text } if !text.is_empty() => {
                        Some(InlineContent::Text { text: text.clone() })
                    }
                    ParsedNode::Link { text, href } if !text.is_empty() => {
                        Some(InlineContent::Link {
                            text: text.clone(),
                            href: href.clone(),
                        })
                    }
                    _ => None,
                })
                .collect();
            if !inline.is_empty() {
                blocks.push(Block::RichParagraph { segments: inline });
            }
        }
        ParsedNode::Quote { text } => {
            blocks.push(Block::Quote {
                text: text.clone(),
                attribution: None,
            });
        }
        ParsedNode::CodeBlock { code, language } => {
            blocks.push(Block::CodeBlock {
                code: code.clone(),
                language: language.clone(),
            });
        }
        ParsedNode::ImageRef { src, alt } => {
            blocks.push(Block::ImageReference {
                src: src.clone(),
                alt: alt.clone(),
                width: None,
                height: None,
            });
        }
        ParsedNode::Link { text, href } => {
            blocks.push(Block::Link {
                text: text.clone(),
                href: href.clone(),
            });
        }
        ParsedNode::Table { rows } => {
            // Detect layout tables: 1 column means it's used for layout, not data.
            // Flatten cells into top-level blocks so they word-wrap and render correctly.
            let max_cols = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
            if max_cols <= 1 {
                for row in rows {
                    for cell_nodes in &row.cells {
                        for node in cell_nodes {
                            convert_node(node, blocks, outline);
                        }
                    }
                }
            } else {
                blocks.push(convert_table(rows));
            }
        }
        ParsedNode::Divider => {
            blocks.push(Block::Divider);
        }
        ParsedNode::ListItem { text, .. } => {
            blocks.push(Block::Paragraph { text: text.clone() });
        }
    }
}

fn convert_node_to_block(node: &ParsedNode) -> Block {
    match node {
        ParsedNode::ListItem {
            text,
            href,
            children,
        } => {
            let child_blocks: Vec<Block> = children.iter().map(convert_node_to_block).collect();
            Block::ListItem {
                text: text.clone(),
                href: href.clone(),
                children: child_blocks,
            }
        }
        ParsedNode::List { ordered, items } => Block::List {
            ordered: *ordered,
            items: items.iter().map(convert_node_to_block).collect(),
        },
        ParsedNode::Paragraph { text } => Block::Paragraph { text: text.clone() },
        ParsedNode::Text { text } => Block::Text { text: text.clone() },
        ParsedNode::Heading { level, text } => Block::Heading {
            level: *level,
            text: text.clone(),
            id: Some(slugify(text)),
        },
        ParsedNode::Link { text, href } => Block::Link {
            text: text.clone(),
            href: href.clone(),
        },
        ParsedNode::Quote { text } => Block::Quote {
            text: text.clone(),
            attribution: None,
        },
        ParsedNode::CodeBlock { code, language } => Block::CodeBlock {
            code: code.clone(),
            language: language.clone(),
        },
        ParsedNode::ImageRef { src, alt } => Block::ImageReference {
            src: src.clone(),
            alt: alt.clone(),
            width: None,
            height: None,
        },
        ParsedNode::RichParagraph { segments } => {
            let inline: Vec<InlineContent> = segments
                .iter()
                .filter_map(|s| match s {
                    ParsedNode::Text { text } if !text.is_empty() => {
                        Some(InlineContent::Text { text: text.clone() })
                    }
                    ParsedNode::Link { text, href } if !text.is_empty() => {
                        Some(InlineContent::Link {
                            text: text.clone(),
                            href: href.clone(),
                        })
                    }
                    _ => None,
                })
                .collect();
            if inline.is_empty() {
                Block::Paragraph {
                    text: String::new(),
                }
            } else {
                Block::RichParagraph { segments: inline }
            }
        }
        ParsedNode::Table { rows } => convert_table(rows),
        ParsedNode::Divider => Block::Divider,
    }
}

fn convert_table(rows: &[TableRow]) -> Block {
    let row_blocks = rows
        .iter()
        .map(|r| Block::TableRow {
            cells: r
                .cells
                .iter()
                .map(|cell_nodes| Block::TableCell {
                    content: cell_nodes.iter().map(convert_node_to_block).collect(),
                })
                .collect(),
            is_header: r.is_header,
        })
        .collect();
    Block::Table { rows: row_blocks }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapt_simple_article() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test Article</title></head>
<body>
<main>
  <h1>Hello World</h1>
  <p>This is a test paragraph with enough content to pass the threshold.</p>
  <p>Another paragraph to make the content substantial enough for extraction.</p>
</main>
</body>
</html>"#;
        let url = Url::parse("https://example.org/article").unwrap();
        let result = adapt_html(html, &url);
        match result {
            AdapterResult::Success(doc) | AdapterResult::PartialSuccess(doc) => {
                assert!(!doc.blocks.is_empty());
                assert!(!doc.outline.is_empty());
                assert_eq!(doc.metadata.title, "Test Article");
            }
            AdapterResult::Unsupported(reason) => panic!("Unsupported: {reason}"),
            AdapterResult::Failed(e) => panic!("Failed: {e}"),
        }
    }

    #[test]
    fn empty_page_returns_unsupported() {
        let html = r#"<!DOCTYPE html><html><head><title>App</title></head><body></body></html>"#;
        let url = Url::parse("https://example.org/spa").unwrap();
        let result = adapt_html(html, &url);
        assert!(matches!(result, AdapterResult::Unsupported(_)));
    }
}
