use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const MODEL_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationDocument {
    pub id: String,
    pub model_version: String,
    pub source: SourceInfo,
    pub metadata: DocumentMetadata,
    pub blocks: Vec<Block>,
    pub outline: Vec<OutlineItem>,
    pub audit: NetworkAudit,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub requested_url: String,
    pub final_url: String,
    pub retrieved_at: DateTime<Utc>,
    pub content_type: String,
    #[serde(default = "default_utf8")]
    pub charset: String,
    #[serde(default = "default_charset_source")]
    pub charset_source: String,
}

fn default_utf8() -> String {
    "utf-8".to_string()
}
fn default_charset_source() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub title: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub canonical_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NetworkAudit {
    pub total_requests: u32,
    pub total_bytes: u64,
    pub redirect_count: u32,
    pub blocked_count: u32,
    pub scripts_discovered: u32,
    pub scripts_executed: u32,
    pub third_party_requests: u32,
    pub adapter_name: String,
    pub adapter_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub block_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Heading {
        level: u8,
        text: String,
        id: Option<String>,
    },
    Paragraph {
        text: String,
    },
    RichParagraph {
        segments: Vec<InlineContent>,
    },
    Text {
        text: String,
    },
    List {
        ordered: bool,
        items: Vec<Block>,
    },
    ListItem {
        text: String,
        href: Option<String>,
        children: Vec<Block>,
    },
    Quote {
        text: String,
        attribution: Option<String>,
    },
    CodeBlock {
        code: String,
        language: Option<String>,
    },
    ImageReference {
        src: String,
        alt: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Table {
        rows: Vec<Block>,
    },
    TableRow {
        cells: Vec<Block>,
        is_header: bool,
    },
    TableCell {
        content: Vec<Block>,
    },
    Divider,
    Notice {
        kind: NoticeKind,
        text: String,
    },
    Link {
        text: String,
        href: String,
    },
    LinkList {
        items: Vec<Block>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineContent {
    Text { text: String },
    Link { text: String, href: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoticeKind {
    Info,
    Warning,
    Error,
    Unsupported,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("model_version is missing")]
    MissingModelVersion,
    #[error("title is empty")]
    EmptyTitle,
    #[error("requested_url is empty")]
    EmptyRequestedUrl,
    #[error("block contains raw HTML: {0}")]
    BlockContainsHtml(String),
    #[error("invalid block nesting in {0}")]
    InvalidNesting(String),
}

impl InformationDocument {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.model_version.is_empty() {
            return Err(ValidationError::MissingModelVersion);
        }
        if self.source.requested_url.is_empty() {
            return Err(ValidationError::EmptyRequestedUrl);
        }
        for block in &self.blocks {
            validate_block(block)?;
        }
        Ok(())
    }
}

fn contains_html(s: &str) -> bool {
    s.contains('<') && s.contains('>')
}

fn validate_block(block: &Block) -> Result<(), ValidationError> {
    match block {
        Block::RichParagraph { segments } => {
            for seg in segments {
                match seg {
                    InlineContent::Text { text } | InlineContent::Link { text, .. } => {
                        if contains_html(text) {
                            return Err(ValidationError::BlockContainsHtml(
                                text.chars().take(40).collect(),
                            ));
                        }
                    }
                }
            }
        }
        Block::Heading { text, .. } | Block::Paragraph { text } | Block::Text { text } => {
            if contains_html(text) {
                return Err(ValidationError::BlockContainsHtml(
                    text.chars().take(40).collect(),
                ));
            }
        }
        Block::Quote { text, .. } => {
            if contains_html(text) {
                return Err(ValidationError::BlockContainsHtml(
                    text.chars().take(40).collect(),
                ));
            }
        }
        Block::CodeBlock { .. } => {}
        Block::List { items, .. } => {
            for item in items {
                validate_block(item)?;
            }
        }
        Block::ListItem { text, children, .. } => {
            if contains_html(text) {
                return Err(ValidationError::BlockContainsHtml(
                    text.chars().take(40).collect(),
                ));
            }
            for child in children {
                validate_block(child)?;
            }
        }
        Block::Table { rows } => {
            for row in rows {
                match row {
                    Block::TableRow { cells, .. } => {
                        for cell in cells {
                            validate_block(cell)?;
                        }
                    }
                    _ => return Err(ValidationError::InvalidNesting("Table".into())),
                }
            }
        }
        Block::TableRow { cells, .. } => {
            for cell in cells {
                validate_block(cell)?;
            }
        }

        Block::TableCell { content } => {
            for block in content {
                validate_block(block)?;
            }
        }
        Block::Notice { text, .. } => {
            if contains_html(text) {
                return Err(ValidationError::BlockContainsHtml(
                    text.chars().take(40).collect(),
                ));
            }
        }
        Block::Link { text, href } => {
            if contains_html(text) || contains_html(href) {
                return Err(ValidationError::BlockContainsHtml(
                    text.chars().take(40).collect(),
                ));
            }
        }
        Block::LinkList { items } => {
            for item in items {
                validate_block(item)?;
            }
        }
        Block::ImageReference { .. } | Block::Divider => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn minimal_doc() -> InformationDocument {
        InformationDocument {
            id: "test-id".into(),
            model_version: MODEL_VERSION.into(),
            source: SourceInfo {
                requested_url: "https://example.org".into(),
                final_url: "https://example.org".into(),
                retrieved_at: Utc::now(),
                content_type: "text/html".into(),
                charset: "utf-8".into(),
                charset_source: "default".into(),
            },
            metadata: DocumentMetadata {
                title: "Test".into(),
                ..Default::default()
            },
            blocks: vec![],
            outline: vec![],
            audit: NetworkAudit::default(),
            warnings: vec![],
        }
    }

    #[test]
    fn valid_empty_doc() {
        assert!(minimal_doc().validate().is_ok());
    }

    #[test]
    fn missing_model_version() {
        let mut doc = minimal_doc();
        doc.model_version = String::new();
        assert!(matches!(
            doc.validate(),
            Err(ValidationError::MissingModelVersion)
        ));
    }

    #[test]
    fn block_with_html_rejected() {
        let mut doc = minimal_doc();
        doc.blocks.push(Block::Paragraph {
            text: "<script>alert(1)</script>".into(),
        });
        assert!(matches!(
            doc.validate(),
            Err(ValidationError::BlockContainsHtml(_))
        ));
    }

    #[test]
    fn valid_heading_block() {
        let mut doc = minimal_doc();
        doc.blocks.push(Block::Heading {
            level: 1,
            text: "Hello World".into(),
            id: None,
        });
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn serialization_round_trip() {
        let doc = minimal_doc();
        let json = serde_json::to_string(&doc).unwrap();
        let doc2: InformationDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.model_version, doc2.model_version);
        assert_eq!(doc.source.requested_url, doc2.source.requested_url);
    }

    #[test]
    fn charset_defaults_on_old_doc() {
        // Documents saved before charset fields were added should deserialize with UTF-8 defaults.
        let json = r#"{
            "id":"x","model_version":"0.1.0",
            "source":{"requested_url":"https://example.org","final_url":"https://example.org",
                      "retrieved_at":"2026-01-01T00:00:00Z","content_type":"text/html"},
            "metadata":{"title":"T"},"blocks":[],"outline":[],"audit":{},"warnings":[]
        }"#;
        let doc: InformationDocument = serde_json::from_str(json).unwrap();
        assert_eq!(doc.source.charset, "utf-8");
        assert_eq!(doc.source.charset_source, "default");
    }

    #[test]
    fn invalid_json_rejected() {
        let result: Result<InformationDocument, _> = serde_json::from_str("{not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn wrong_model_version_detectable() {
        let mut doc = minimal_doc();
        doc.model_version = "99.0.0".into();
        // Validation doesn't check the version value itself — caller checks it
        let json = serde_json::to_string(&doc).unwrap();
        let doc2: InformationDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc2.model_version, "99.0.0");
    }
}
