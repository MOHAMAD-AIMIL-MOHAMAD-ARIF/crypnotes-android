use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub use crypnotes_versioning::DOC_SCHEMA_VERSION;
pub use crypnotes_versioning::NOTE_PAYLOAD_SCHEMA_VERSION;

pub const NOTE_CHAR_LIMIT: usize = 20_000;
const ALLOWED_BLOCK_TYPES: &[&str] = &[
    "paragraph",
    "checklist_item",
    "bullet_item",
    "numbered_item",
    "heading1",
    "heading2",
    "heading3",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDocument {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("invalid JSON document: {0}")]
    InvalidJson(String),
    #[error("document must be an object with a blocks array")]
    InvalidStructure,
    #[error("unsupported block type: {0}")]
    InvalidBlockType(String),
    #[error("note exceeds max length of {NOTE_CHAR_LIMIT} characters")]
    NoteTooLarge,
}

pub fn validate_note_document(canonical_json: &str) -> Result<(), DocumentError> {
    let value: Value = serde_json::from_str(canonical_json)
        .map_err(|err| DocumentError::InvalidJson(err.to_string()))?;

    let blocks = value
        .get("blocks")
        .and_then(Value::as_array)
        .ok_or(DocumentError::InvalidStructure)?;

    for block in blocks {
        let block_type = block
            .get("type")
            .and_then(Value::as_str)
            .ok_or(DocumentError::InvalidStructure)?;

        if !ALLOWED_BLOCK_TYPES.contains(&block_type) {
            return Err(DocumentError::InvalidBlockType(block_type.to_owned()));
        }

        if block.get("text").and_then(Value::as_str).is_none() {
            return Err(DocumentError::InvalidStructure);
        }
    }

    Ok(())
}

pub fn validate_note_char_limit(plain_text: &str) -> Result<u32, DocumentError> {
    let count = plain_text.chars().count();
    if count > NOTE_CHAR_LIMIT {
        return Err(DocumentError::NoteTooLarge);
    }
    Ok(count as u32)
}

pub fn derive_display_title(
    explicit_title: &str,
    canonical_json: &str,
) -> Result<String, DocumentError> {
    if !explicit_title.trim().is_empty() {
        return Ok(explicit_title.to_owned());
    }

    let doc = parse_doc(canonical_json)?;

    if let Some(paragraph_title) = doc
        .blocks
        .iter()
        .find(|b| b.block_type == "paragraph" && !b.text.trim().is_empty())
        .map(|b| b.text.trim().to_owned())
    {
        return Ok(paragraph_title);
    }

    let fallback = doc
        .blocks
        .iter()
        .find_map(|b| {
            let text = b.text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_owned())
            }
        })
        .unwrap_or_default();

    Ok(fallback)
}

pub fn convert_text_to_checklist(canonical_json: &str) -> Result<String, DocumentError> {
    let mut doc = parse_doc(canonical_json)?;
    for block in &mut doc.blocks {
        block.block_type = "checklist_item".to_owned();
        block.checked = Some(false);
    }
    serde_json::to_string(&doc).map_err(|err| DocumentError::InvalidJson(err.to_string()))
}

pub fn convert_checklist_to_text(canonical_json: &str) -> Result<String, DocumentError> {
    let mut doc = parse_doc(canonical_json)?;
    for block in &mut doc.blocks {
        block.block_type = "paragraph".to_owned();
        block.checked = None;
    }
    serde_json::to_string(&doc).map_err(|err| DocumentError::InvalidJson(err.to_string()))
}

fn parse_doc(canonical_json: &str) -> Result<CanonicalDocument, DocumentError> {
    validate_note_document(canonical_json)?;
    serde_json::from_str(canonical_json).map_err(|err| DocumentError::InvalidJson(err.to_string()))
}

pub fn current_doc_schema_version() -> u32 {
    DOC_SCHEMA_VERSION
}

pub fn current_note_payload_schema_version() -> u32 {
    NOTE_PAYLOAD_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> String {
        serde_json::json!({
            "blocks": [
                {"type": "paragraph", "text": "Top paragraph"},
                {"type": "bullet_item", "text": "Second line"}
            ]
        })
        .to_string()
    }

    #[test]
    fn char_limit_boundaries() {
        let under = "a".repeat(19_999);
        let exact = "a".repeat(20_000);
        let over = "a".repeat(20_001);

        assert_eq!(validate_note_char_limit(&under).unwrap(), 19_999);
        assert_eq!(validate_note_char_limit(&exact).unwrap(), 20_000);
        assert!(matches!(
            validate_note_char_limit(&over),
            Err(DocumentError::NoteTooLarge)
        ));
    }

    #[test]
    fn title_derives_from_first_paragraph_when_explicit_empty() {
        let title = derive_display_title("", &sample_doc()).unwrap();
        assert_eq!(title, "Top paragraph");
    }

    #[test]
    fn checklist_conversion_is_reversible_for_text() {
        let doc = sample_doc();
        let checklist = convert_text_to_checklist(&doc).unwrap();
        let reverted = convert_checklist_to_text(&checklist).unwrap();

        let reverted_doc: CanonicalDocument = serde_json::from_str(&reverted).unwrap();
        assert_eq!(reverted_doc.blocks[0].block_type, "paragraph");
        assert_eq!(reverted_doc.blocks[1].block_type, "paragraph");
        assert_eq!(reverted_doc.blocks[0].text, "Top paragraph");
        assert_eq!(reverted_doc.blocks[1].text, "Second line");
    }

    #[test]
    fn schema_version_accessors_match_shared_constants() {
        assert_eq!(current_doc_schema_version(), DOC_SCHEMA_VERSION);
        assert_eq!(
            current_note_payload_schema_version(),
            NOTE_PAYLOAD_SCHEMA_VERSION
        );
    }
}
