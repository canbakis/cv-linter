mod spelling;

use serde::Serialize;
use sha2::{Digest, Sha256};
use spelling::{
    DEVELOPER_ALLOWLIST_VERSION, DICTIONARY_SOURCE_COMMIT, SPELLBOOK_VERSION,
    SpellingClassification, check_blocks, normalize_allow_word,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{self, Write as _};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use xberg::types::document_structure::NodeContent;
use xberg::{ExtractInput, ExtractionConfig, PageConfig, SecurityLimits};

pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_TEXT_LINES: usize = 100_000;
pub const MAX_DOCUMENT_BLOCKS: usize = 10_000;
pub const MAX_FINDINGS: usize = 10_000;
pub const MAX_DOCUMENT_PAGES: usize = 100;
pub const MAX_ALLOW_WORDS: usize = 256;
pub const MAX_ALLOW_WORD_BYTES: usize = 360;
const MAX_EXTRACTION_WARNINGS: usize = 100;
const MAX_WARNING_MESSAGE_BYTES: usize = 2_000;
const MIN_RECOGNIZED_SECTION_HEADINGS: usize = 2;
const RECOGNIZED_SECTION_HEADINGS: &[&str] = &[
    "summary",
    "professional summary",
    "profile",
    "professional profile",
    "core competencies",
    "key competencies",
    "competencies",
    "experience",
    "work experience",
    "professional experience",
    "employment history",
    "technical skills",
    "skills",
    "projects",
    "selected projects",
    "education",
    "certifications",
    "languages",
    "volunteer experience",
    "publications",
    "awards",
    "outside work",
    "interests",
    "sammanfattning",
    "profil",
    "kärnkompetenser",
    "nyckelkompetenser",
    "kompetenser",
    "erfarenhet",
    "arbetslivserfarenhet",
    "yrkeserfarenhet",
    "tekniska färdigheter",
    "tekniska kompetenser",
    "färdigheter",
    "projekt",
    "utvalda projekt",
    "utbildning",
    "certifieringar",
    "språk",
    "ideellt arbete",
    "volontärarbete",
    "publikationer",
    "utmärkelser",
    "fritidsintressen",
];
pub const SCHEMA_VERSION: &str = "0.3.0";
pub const RULESET_VERSION: &str = "0.2.2";
pub const XBERG_VERSION: &str = "1.1.5";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    InputTooLarge {
        actual: usize,
        limit: usize,
    },
    InvalidUtf8 {
        valid_up_to: usize,
    },
    TooManyLines {
        limit: usize,
    },
    TooManyBlocks {
        limit: usize,
    },
    TooManyFindings {
        limit: usize,
    },
    TooManyAllowWords {
        actual: usize,
        limit: usize,
    },
    InvalidAllowWord {
        index: usize,
        message: &'static str,
    },
    UnsupportedFormat {
        extension: String,
    },
    ExtractionFailed {
        format: &'static str,
        message: String,
    },
}

impl fmt::Display for ExtractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { actual, limit } => {
                write!(formatter, "input is {actual} bytes; limit is {limit} bytes")
            }
            Self::InvalidUtf8 { valid_up_to } => write!(
                formatter,
                "input is not valid UTF-8 text (first invalid byte at offset {valid_up_to})"
            ),
            Self::TooManyLines { limit } => {
                write!(formatter, "input exceeds the {limit}-line text limit")
            }
            Self::TooManyBlocks { limit } => {
                write!(
                    formatter,
                    "extraction exceeds the {limit}-block output limit"
                )
            }
            Self::TooManyFindings { limit } => {
                write!(formatter, "lint exceeds the {limit}-finding limit")
            }
            Self::TooManyAllowWords { actual, limit } => write!(
                formatter,
                "lint received {actual} allow words; limit is {limit}"
            ),
            Self::InvalidAllowWord { index, message } => {
                write!(formatter, "allow word {} is invalid: {message}", index + 1)
            }
            Self::UnsupportedFormat { extension } => write!(
                formatter,
                "unsupported input format '{extension}'; expected PDF, DOCX, or Markdown"
            ),
            Self::ExtractionFailed { format, message } => {
                write!(formatter, "failed to extract {format} input: {message}")
            }
        }
    }
}

impl std::error::Error for ExtractError {}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InputIdentity {
    pub sha256: String,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtractorIdentity {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RulesetIdentity {
    pub version: &'static str,
    pub spelling: SpellingRulesetIdentity,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SpellingRulesetIdentity {
    pub engine: &'static str,
    pub engine_version: &'static str,
    pub dictionary_source: &'static str,
    pub dictionary_revision: &'static str,
    pub developer_allowlist_version: &'static str,
    pub matching: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LintOptionsIdentity {
    pub spelling_allow_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Ok,
    NoReadableText,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BoundingBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceLocator {
    PlainText {
        line: usize,
        start_byte: usize,
        end_byte: usize,
    },
    Markdown {
        line: Option<usize>,
        start_byte: Option<usize>,
        end_byte: Option<usize>,
        ordered_block: usize,
    },
    Pdf {
        page: Option<u32>,
        bounding_box: Option<BoundingBox>,
        ordered_block: usize,
    },
    Docx {
        element: usize,
        table: Option<usize>,
        row: Option<usize>,
        column: Option<usize>,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtractionWarning {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TextBlock {
    pub id: String,
    pub kind: &'static str,
    pub text: String,
    pub source: SourceLocator,
    pub provenance: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ExtractedDocument {
    pub schema_version: &'static str,
    pub status: DocumentStatus,
    pub input: InputIdentity,
    pub extractor: ExtractorIdentity,
    pub blocks: Vec<TextBlock>,
    pub warnings: Vec<ExtractionWarning>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FindingEvidence {
    pub block_id: Option<String>,
    pub source: Option<SourceLocator>,
    pub block_start_byte: Option<usize>,
    pub block_end_byte: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub message: String,
    pub evidence: Option<FindingEvidence>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LintReport {
    pub schema_version: &'static str,
    pub status: DocumentStatus,
    pub input: InputIdentity,
    pub extractor: ExtractorIdentity,
    pub ruleset: RulesetIdentity,
    pub options: LintOptionsIdentity,
    pub block_count: usize,
    pub warnings: Vec<ExtractionWarning>,
    pub findings: Vec<Finding>,
    pub spelling_findings: Vec<Finding>,
}

pub fn read_selected_file(path: &Path) -> Result<Vec<u8>, io::Error> {
    let metadata = path.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected input is not a regular file",
        ));
    }

    if metadata.len() > MAX_INPUT_BYTES as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "selected input is {} bytes; limit is {MAX_INPUT_BYTES} bytes",
                metadata.len()
            ),
        ));
    }

    let mut input = Vec::with_capacity(metadata.len() as usize);
    File::open(path)?
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut input)?;

    if input.len() > MAX_INPUT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("selected input exceeds the {MAX_INPUT_BYTES}-byte limit"),
        ));
    }

    Ok(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentFormat {
    PlainText,
    Markdown,
    Pdf,
    Docx,
}

impl DocumentFormat {
    fn from_path(path: &Path) -> Result<Self, ExtractError> {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match extension.as_str() {
            "txt" => Ok(Self::PlainText),
            "md" | "markdown" => Ok(Self::Markdown),
            "pdf" => Ok(Self::Pdf),
            "docx" => Ok(Self::Docx),
            _ => Err(ExtractError::UnsupportedFormat {
                extension: if extension.is_empty() {
                    "<none>".to_owned()
                } else {
                    extension
                },
            }),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::PlainText => "plain text",
            Self::Markdown => "Markdown",
            Self::Pdf => "PDF",
            Self::Docx => "DOCX",
        }
    }

    fn mime_type(self) -> &'static str {
        match self {
            Self::PlainText => "text/plain",
            Self::Markdown => "text/markdown",
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        }
    }
}

/// Extract one explicitly selected document through the project-owned format adapter.
///
/// PDF, DOCX, and Markdown are parsed by the exactly pinned Xberg dependency. The
/// plain-text branch is retained for internal tests and debugging, not as a marketed
/// input format.
pub async fn extract_document(
    path: &Path,
    input: &[u8],
) -> Result<ExtractedDocument, ExtractError> {
    validate_size(input)?;
    let format = DocumentFormat::from_path(path)?;
    if format == DocumentFormat::PlainText {
        return extract_plain_text(input);
    }
    if format == DocumentFormat::Markdown {
        let text = std::str::from_utf8(input).map_err(|error| ExtractError::InvalidUtf8 {
            valid_up_to: error.valid_up_to(),
        })?;
        if lines_with_offsets(text).nth(MAX_DOCUMENT_BLOCKS).is_some() {
            return Err(ExtractError::TooManyLines {
                limit: MAX_DOCUMENT_BLOCKS,
            });
        }
    }

    XbergDocumentExtractor.extract(path, input, format).await
}

/// Run deterministic checks after extracting one explicitly selected document.
pub async fn lint_document(path: &Path, input: &[u8]) -> Result<LintReport, ExtractError> {
    lint_document_with_allow_words(path, input, &[]).await
}

/// Run deterministic checks with an operation-scoped spelling allowlist.
///
/// Allow words are used only for this call and are never persisted or added to
/// the bundled dictionaries.
pub async fn lint_document_with_allow_words(
    path: &Path,
    input: &[u8],
    allow_words: &[String],
) -> Result<LintReport, ExtractError> {
    validate_allow_words(allow_words)?;
    let format = DocumentFormat::from_path(path)?;
    if format == DocumentFormat::PlainText {
        return lint_plain_text_with_allow_words(input, allow_words);
    }

    lint_extracted_document(extract_document(path, input).await?, allow_words)
}

struct XbergDocumentExtractor;

impl XbergDocumentExtractor {
    async fn extract(
        &self,
        path: &Path,
        input: &[u8],
        format: DocumentFormat,
    ) -> Result<ExtractedDocument, ExtractError> {
        let config = ExtractionConfig {
            use_cache: false,
            enable_quality_processing: false,
            disable_ocr: true,
            extraction_timeout_secs: Some(30),
            max_embedded_file_bytes: Some(MAX_INPUT_BYTES as u64),
            security_limits: Some(SecurityLimits {
                max_archive_size: 50 * 1024 * 1024,
                max_content_size: MAX_INPUT_BYTES,
                max_files_in_archive: 1_000,
                max_nesting_depth: 128,
                max_iterations: 1_000_000,
                max_xml_depth: 128,
                max_table_cells: MAX_DOCUMENT_BLOCKS,
                max_pages: Some(MAX_DOCUMENT_PAGES),
                ..SecurityLimits::default()
            }),
            pages: Some(PageConfig {
                extract_pages: true,
                ..Default::default()
            }),
            include_document_structure: true,
            ..Default::default()
        };
        let filename = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned());
        let envelope = xberg::extract(
            ExtractInput::from_bytes(input.to_vec(), format.mime_type(), filename),
            &config,
        )
        .await
        .map_err(|error| ExtractError::ExtractionFailed {
            format: format.label(),
            message: error.to_string(),
        })?;

        if let Some(error) = envelope.errors.first() {
            return Err(ExtractError::ExtractionFailed {
                format: format.label(),
                message: error.message.clone(),
            });
        }

        let extracted =
            envelope
                .results
                .into_iter()
                .next()
                .ok_or_else(|| ExtractError::ExtractionFailed {
                    format: format.label(),
                    message: "the extractor returned no document result".to_owned(),
                })?;

        adapt_xberg_document(format, input, extracted)
    }
}

fn adapt_xberg_document(
    format: DocumentFormat,
    input: &[u8],
    extracted: xberg::ExtractedDocument,
) -> Result<ExtractedDocument, ExtractError> {
    let mut blocks = Vec::new();
    let mut markdown_cursor = 0;
    let mut table_number = 0;

    if let Some(structure) = &extracted.document {
        for (node_index, node) in structure.nodes.iter().enumerate() {
            match &node.content {
                NodeContent::Title { text } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "title",
                    text,
                    None,
                    None,
                ),
                NodeContent::Heading { text, .. } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "heading",
                    text,
                    None,
                    None,
                ),
                NodeContent::Paragraph { text } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "paragraph",
                    text,
                    None,
                    None,
                ),
                NodeContent::ListItem { text } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "list_item",
                    text,
                    None,
                    None,
                ),
                NodeContent::Code { text, .. } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "code",
                    text,
                    None,
                    None,
                ),
                NodeContent::Formula { text } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "formula",
                    text,
                    None,
                    None,
                ),
                NodeContent::Footnote { text } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "footnote",
                    text,
                    None,
                    None,
                ),
                NodeContent::Comment { text } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "comment",
                    text,
                    None,
                    None,
                ),
                NodeContent::DefinitionItem { term, definition } => {
                    let text = format!("{term}: {definition}");
                    push_xberg_block(
                        &mut blocks,
                        format,
                        input,
                        &mut markdown_cursor,
                        node_index,
                        node,
                        "definition_item",
                        &text,
                        None,
                        None,
                    );
                }
                NodeContent::Citation { text, .. } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "citation",
                    text,
                    None,
                    None,
                ),
                NodeContent::RawBlock { content, .. } => push_xberg_block(
                    &mut blocks,
                    format,
                    input,
                    &mut markdown_cursor,
                    node_index,
                    node,
                    "raw_block",
                    content,
                    None,
                    None,
                ),
                NodeContent::Table { grid } => {
                    table_number += 1;
                    let mut rows = BTreeMap::<_, Vec<_>>::new();
                    for cell in &grid.cells {
                        let text = cell.content.trim();
                        if !text.is_empty() {
                            rows.entry(cell.row).or_default().push((cell.col, text));
                        }
                    }
                    for (row, mut cells) in rows {
                        cells.sort_unstable_by_key(|(column, _)| *column);
                        let text = cells
                            .into_iter()
                            .map(|(_, text)| text)
                            .collect::<Vec<_>>()
                            .join(" | ");
                        push_xberg_block(
                            &mut blocks,
                            format,
                            input,
                            &mut markdown_cursor,
                            node_index,
                            node,
                            "table_row",
                            &text,
                            Some(table_number),
                            Some(row as usize + 1),
                        );
                        ensure_block_limit(&blocks)?;
                    }
                }
                NodeContent::Image { .. }
                | NodeContent::List { .. }
                | NodeContent::Quote
                | NodeContent::Group { .. }
                | NodeContent::PageBreak
                | NodeContent::Slide { .. }
                | NodeContent::DefinitionList
                | NodeContent::Admonition { .. }
                | NodeContent::MetadataBlock { .. } => {}
            }
            ensure_block_limit(&blocks)?;
        }
    }

    if blocks.is_empty() && !extracted.content.trim().is_empty() {
        push_fallback_blocks(&mut blocks, format, input, &extracted)?;
    }

    let warning_count = extracted.processing_warnings.len();
    let mut warnings = extracted
        .processing_warnings
        .iter()
        .take(MAX_EXTRACTION_WARNINGS)
        .map(|warning| ExtractionWarning {
            code: "extractor.warning",
            message: bounded_warning_message(&warning.source, &warning.message),
        })
        .collect::<Vec<_>>();
    if warning_count > MAX_EXTRACTION_WARNINGS {
        warnings.push(ExtractionWarning {
            code: "extractor.warnings_truncated",
            message: format!(
                "The extractor returned {warning_count} warnings; only the first {MAX_EXTRACTION_WARNINGS} are included."
            ),
        });
    }
    let status = if blocks.is_empty() {
        warnings.push(ExtractionWarning {
            code: "extraction.no_native_text",
            message: if extracted.counts.images > 0 {
                "No readable native text was extracted. The document contains images and may be scanned or image-only; OCR is not included in this MVP.".to_owned()
            } else {
                "No readable native text was extracted; OCR is not included in this MVP.".to_owned()
            },
        });
        DocumentStatus::NoReadableText
    } else {
        DocumentStatus::Ok
    };

    Ok(ExtractedDocument {
        schema_version: SCHEMA_VERSION,
        status,
        input: input_identity(input),
        extractor: ExtractorIdentity {
            name: "xberg",
            version: XBERG_VERSION,
        },
        blocks,
        warnings,
    })
}

fn ensure_block_limit(blocks: &[TextBlock]) -> Result<(), ExtractError> {
    if blocks.len() > MAX_DOCUMENT_BLOCKS {
        return Err(ExtractError::TooManyBlocks {
            limit: MAX_DOCUMENT_BLOCKS,
        });
    }
    Ok(())
}

fn bounded_warning_message(source: &str, message: &str) -> String {
    let combined = format!("{source}: {message}");
    if combined.len() <= MAX_WARNING_MESSAGE_BYTES {
        return combined;
    }

    let mut end = MAX_WARNING_MESSAGE_BYTES;
    while !combined.is_char_boundary(end) {
        end -= 1;
    }
    let mut truncated = combined[..end].to_owned();
    truncated.push('…');
    truncated
}

#[allow(clippy::too_many_arguments)]
fn push_xberg_block(
    blocks: &mut Vec<TextBlock>,
    format: DocumentFormat,
    input: &[u8],
    markdown_cursor: &mut usize,
    node_index: usize,
    node: &xberg::types::document_structure::DocumentNode,
    kind: &'static str,
    text: &str,
    table: Option<usize>,
    row: Option<usize>,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let ordered_block = blocks.len() + 1;
    let source = match format {
        DocumentFormat::Markdown => {
            let source_text = std::str::from_utf8(input).expect("Markdown UTF-8 was validated");
            let span = locate_markdown_text(source_text, text, *markdown_cursor);
            if let Some((start, end)) = span {
                *markdown_cursor = end;
                SourceLocator::Markdown {
                    line: Some(line_number_at_offset(source_text, start)),
                    start_byte: Some(start),
                    end_byte: Some(end),
                    ordered_block,
                }
            } else {
                SourceLocator::Markdown {
                    line: None,
                    start_byte: None,
                    end_byte: None,
                    ordered_block,
                }
            }
        }
        DocumentFormat::Pdf => SourceLocator::Pdf {
            page: node.page,
            bounding_box: node.bbox.map(|bbox| BoundingBox {
                x0: bbox.x0,
                y0: bbox.y0,
                x1: bbox.x1,
                y1: bbox.y1,
            }),
            ordered_block,
        },
        DocumentFormat::Docx => SourceLocator::Docx {
            element: node_index + 1,
            table,
            row,
            column: None,
        },
        DocumentFormat::PlainText => unreachable!("plain text does not use Xberg"),
    };

    blocks.push(TextBlock {
        id: format!("block-{ordered_block:04}"),
        kind,
        text: text.to_owned(),
        source,
        provenance: "native_text",
    });
}

fn push_fallback_blocks(
    blocks: &mut Vec<TextBlock>,
    format: DocumentFormat,
    input: &[u8],
    extracted: &xberg::ExtractedDocument,
) -> Result<(), ExtractError> {
    let mut markdown_cursor = 0;
    if format == DocumentFormat::Pdf
        && let Some(pages) = &extracted.pages
    {
        for page in pages {
            for text in page.content.split("\n\n") {
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                let ordered_block = blocks.len() + 1;
                blocks.push(TextBlock {
                    id: format!("block-{ordered_block:04}"),
                    kind: "paragraph",
                    text: text.to_owned(),
                    source: SourceLocator::Pdf {
                        page: Some(page.page_number),
                        bounding_box: None,
                        ordered_block,
                    },
                    provenance: "native_text",
                });
                ensure_block_limit(blocks)?;
            }
        }
        if !blocks.is_empty() {
            return Ok(());
        }
    }

    for (index, text) in extracted.content.split("\n\n").enumerate() {
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let ordered_block = blocks.len() + 1;
        let source = match format {
            DocumentFormat::Markdown => {
                let source_text = std::str::from_utf8(input).expect("Markdown UTF-8 was validated");
                let span = locate_markdown_text(source_text, text, markdown_cursor);
                if let Some((start, end)) = span {
                    markdown_cursor = end;
                    SourceLocator::Markdown {
                        line: Some(line_number_at_offset(source_text, start)),
                        start_byte: Some(start),
                        end_byte: Some(end),
                        ordered_block,
                    }
                } else {
                    SourceLocator::Markdown {
                        line: None,
                        start_byte: None,
                        end_byte: None,
                        ordered_block,
                    }
                }
            }
            DocumentFormat::Pdf => SourceLocator::Pdf {
                page: None,
                bounding_box: None,
                ordered_block,
            },
            DocumentFormat::Docx => SourceLocator::Docx {
                element: index + 1,
                table: None,
                row: None,
                column: None,
            },
            DocumentFormat::PlainText => unreachable!("plain text does not use Xberg"),
        };
        blocks.push(TextBlock {
            id: format!("block-{ordered_block:04}"),
            kind: "paragraph",
            text: text.to_owned(),
            source,
            provenance: "native_text",
        });
        ensure_block_limit(blocks)?;
    }
    Ok(())
}

fn locate_markdown_text(source: &str, text: &str, cursor: usize) -> Option<(usize, usize)> {
    for line in lines_with_offsets(source) {
        if let Some((relative_start, relative_end)) = markdown_line_content_span(line.text, text) {
            let start = line.start_byte + relative_start;
            if start >= cursor {
                return Some((start, line.start_byte + relative_end));
            }
        }
    }
    None
}

fn markdown_line_content_span(line: &str, text: &str) -> Option<(usize, usize)> {
    let leading = line.len() - line.trim_start().len();
    let candidate = line.trim();
    if candidate == text {
        return Some((leading, leading + text.len()));
    }

    let content_with_spacing = strip_markdown_block_prefix(candidate)?;
    let content_leading = content_with_spacing.len() - content_with_spacing.trim_start().len();
    let content = content_with_spacing.trim();
    if content != text {
        return None;
    }
    let start = leading + candidate.len() - content_with_spacing.len() + content_leading;
    Some((start, start + text.len()))
}

fn strip_markdown_block_prefix(line: &str) -> Option<&str> {
    let heading_markers = line.bytes().take_while(|byte| *byte == b'#').count();
    if (1..=6).contains(&heading_markers)
        && line
            .as_bytes()
            .get(heading_markers)
            .is_some_and(u8::is_ascii_whitespace)
    {
        return Some(&line[heading_markers..]);
    }

    for prefix in ["-", "*", "+", ">"] {
        if let Some(rest) = line.strip_prefix(prefix)
            && rest.chars().next().is_some_and(char::is_whitespace)
        {
            return Some(rest);
        }
    }

    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 {
        let rest = &line[digits..];
        for marker in [".", ")"] {
            if let Some(rest) = rest.strip_prefix(marker)
                && rest.chars().next().is_some_and(char::is_whitespace)
            {
                return Some(rest);
            }
        }
    }
    None
}

fn line_number_at_offset(source: &str, offset: usize) -> usize {
    lines_with_offsets(source)
        .find(|line| offset >= line.start_byte && offset <= line.end_byte)
        .map_or(1, |line| line.number)
}

pub fn extract_plain_text(input: &[u8]) -> Result<ExtractedDocument, ExtractError> {
    validate_size(input)?;
    let text = std::str::from_utf8(input).map_err(|error| ExtractError::InvalidUtf8 {
        valid_up_to: error.valid_up_to(),
    })?;

    let mut blocks = Vec::new();
    for (line_index, raw_line) in lines_with_offsets(text).enumerate() {
        if line_index == MAX_TEXT_LINES {
            return Err(ExtractError::TooManyLines {
                limit: MAX_TEXT_LINES,
            });
        }

        let line = strip_initial_bom(raw_line);
        if line.text.trim().is_empty() {
            continue;
        }

        blocks.push(TextBlock {
            id: format!("block-{:04}", blocks.len() + 1),
            kind: "line",
            text: line.text.to_owned(),
            source: SourceLocator::PlainText {
                line: line.number,
                start_byte: line.start_byte,
                end_byte: line.end_byte,
            },
            provenance: "native_text",
        });
    }

    Ok(ExtractedDocument {
        schema_version: SCHEMA_VERSION,
        status: if blocks.is_empty() {
            DocumentStatus::NoReadableText
        } else {
            DocumentStatus::Ok
        },
        input: input_identity(input),
        extractor: plain_text_extractor(),
        blocks,
        warnings: Vec::new(),
    })
}

pub fn lint_plain_text(input: &[u8]) -> Result<LintReport, ExtractError> {
    lint_plain_text_with_allow_words(input, &[])
}

fn lint_plain_text_with_allow_words(
    input: &[u8],
    allow_words: &[String],
) -> Result<LintReport, ExtractError> {
    let document = extract_plain_text(input)?;
    let text = std::str::from_utf8(input).expect("extract_plain_text already validated UTF-8");
    let mut findings = Vec::new();

    if document.status == DocumentStatus::NoReadableText {
        add_finding(
            &mut findings,
            Finding {
                rule_id: "text.empty",
                severity: Severity::Error,
                message: "The document contains no readable text.".to_owned(),
                evidence: None,
            },
        )?;
    }

    let mut blocks = document.blocks.iter().peekable();
    for line in lines_with_offsets(text) {
        let block = if blocks
            .peek()
            .is_some_and(|block| {
                matches!(block.source, SourceLocator::PlainText { line: block_line, .. } if block_line == line.number)
            })
        {
            blocks.next()
        } else {
            None
        };
        let block_id = block.map(|block| block.id.clone());
        let block_source_start = block.and_then(|block| match block.source {
            SourceLocator::PlainText { start_byte, .. } => Some(start_byte),
            _ => None,
        });

        let without_trailing_whitespace = line.text.trim_end_matches(char::is_whitespace);
        if without_trailing_whitespace.len() != line.text.len() {
            add_finding(
                &mut findings,
                Finding {
                    rule_id: "text.trailing_whitespace",
                    severity: Severity::Info,
                    message: "The line has trailing whitespace.".to_owned(),
                    evidence: Some(FindingEvidence {
                        block_id: block_id.clone(),
                        source: Some(SourceLocator::PlainText {
                            line: line.number,
                            start_byte: line.start_byte + without_trailing_whitespace.len(),
                            end_byte: line.end_byte,
                        }),
                        block_start_byte: block_source_start.map(|start| {
                            line.start_byte + without_trailing_whitespace.len() - start
                        }),
                        block_end_byte: block_source_start.map(|start| line.end_byte - start),
                    }),
                },
            )?;
        }

        for (relative_offset, character) in line.text.char_indices() {
            let absolute_offset = line.start_byte + relative_offset;
            let rule = match character {
                '\0' => Some((
                    "text.nul_character",
                    Severity::Error,
                    "The line contains a NUL character.",
                )),
                '\t' => Some((
                    "text.tab_character",
                    Severity::Info,
                    "The line contains a tab character.",
                )),
                '\u{feff}' if absolute_offset == 0 => Some((
                    "text.utf8_bom",
                    Severity::Info,
                    "The document starts with a UTF-8 byte-order mark.",
                )),
                character if is_bidirectional_control(character) => Some((
                    "text.bidirectional_control",
                    Severity::Warning,
                    "The line contains a bidirectional control character; verify it is intentional.",
                )),
                character
                    if unicode_general_category::get_general_category(character)
                        == unicode_general_category::GeneralCategory::Format =>
                {
                    Some((
                        "text.format_character",
                        Severity::Warning,
                        "The line contains an invisible format character; verify it is intentional.",
                    ))
                }
                character if character.is_control() => Some((
                    "text.control_character",
                    Severity::Warning,
                    "The line contains an unexpected control character.",
                )),
                _ => None,
            };

            if let Some((rule_id, severity, message)) = rule {
                add_finding(
                    &mut findings,
                    Finding {
                        rule_id,
                        severity,
                        message: message.to_owned(),
                        evidence: Some(FindingEvidence {
                            block_id: block_id.clone(),
                            source: Some(SourceLocator::PlainText {
                                line: line.number,
                                start_byte: absolute_offset,
                                end_byte: absolute_offset + character.len_utf8(),
                            }),
                            block_start_byte: block_source_start
                                .map(|start| absolute_offset.saturating_sub(start)),
                            block_end_byte: block_source_start.map(|start| {
                                (absolute_offset + character.len_utf8()).saturating_sub(start)
                            }),
                        }),
                    },
                )?;
            }
        }
    }

    let spelling_findings = collect_spelling_findings(
        &document.blocks,
        allow_words,
        MAX_FINDINGS.saturating_sub(findings.len()),
    )?;

    Ok(LintReport {
        schema_version: SCHEMA_VERSION,
        status: document.status,
        input: document.input,
        extractor: document.extractor,
        ruleset: ruleset_identity(),
        options: lint_options_identity(allow_words),
        block_count: document.blocks.len(),
        warnings: document.warnings,
        findings,
        spelling_findings,
    })
}

fn lint_extracted_document(
    document: ExtractedDocument,
    allow_words: &[String],
) -> Result<LintReport, ExtractError> {
    if document.blocks.len() > MAX_DOCUMENT_BLOCKS {
        return Err(ExtractError::TooManyBlocks {
            limit: MAX_DOCUMENT_BLOCKS,
        });
    }

    let mut findings = Vec::new();
    if document.status == DocumentStatus::NoReadableText {
        add_finding(
            &mut findings,
            Finding {
                rule_id: "text.empty",
                severity: Severity::Error,
                message: "The document contains no readable text.".to_owned(),
                evidence: None,
            },
        )?;
    }

    for block in &document.blocks {
        for line in lines_with_offsets(&block.text) {
            let without_trailing_whitespace = line.text.trim_end_matches(char::is_whitespace);
            if without_trailing_whitespace.len() != line.text.len() {
                add_finding(
                    &mut findings,
                    Finding {
                        rule_id: "text.trailing_whitespace",
                        severity: Severity::Info,
                        message: "The extracted block has trailing whitespace.".to_owned(),
                        evidence: Some(FindingEvidence {
                            block_id: Some(block.id.clone()),
                            source: Some(block.source.clone()),
                            block_start_byte: Some(
                                line.start_byte + without_trailing_whitespace.len(),
                            ),
                            block_end_byte: Some(line.end_byte),
                        }),
                    },
                )?;
            }

            for (relative_offset, character) in line.text.char_indices() {
                let offset = line.start_byte + relative_offset;
                let rule = match character {
                    '\0' => Some((
                        "text.nul_character",
                        Severity::Error,
                        "The extracted block contains a NUL character.",
                    )),
                    '\t' => Some((
                        "text.tab_character",
                        Severity::Info,
                        "The extracted block contains a tab character.",
                    )),
                    character if is_bidirectional_control(character) => Some((
                        "text.bidirectional_control",
                        Severity::Warning,
                        "The extracted block contains a bidirectional control character; verify it is intentional.",
                    )),
                    character
                        if unicode_general_category::get_general_category(character)
                            == unicode_general_category::GeneralCategory::Format =>
                    {
                        Some((
                            "text.format_character",
                            Severity::Warning,
                            "The extracted block contains an invisible format character; verify it is intentional.",
                        ))
                    }
                    character if character.is_control() => Some((
                        "text.control_character",
                        Severity::Warning,
                        "The extracted block contains an unexpected control character.",
                    )),
                    _ => None,
                };

                if let Some((rule_id, severity, message)) = rule {
                    add_finding(
                        &mut findings,
                        Finding {
                            rule_id,
                            severity,
                            message: message.to_owned(),
                            evidence: Some(FindingEvidence {
                                block_id: Some(block.id.clone()),
                                source: Some(block.source.clone()),
                                block_start_byte: Some(offset),
                                block_end_byte: Some(offset + character.len_utf8()),
                            }),
                        },
                    )?;
                }
            }
        }

        if block.kind == "paragraph" && block.text.chars().count() > 1_200 {
            add_finding(
                &mut findings,
                Finding {
                    rule_id: "cv.structure.dense_block",
                    severity: Severity::Info,
                    message: "A long paragraph was extracted as one block; verify that the section is easy to scan.".to_owned(),
                    evidence: Some(block_evidence(block)),
                },
            )?;
        }
    }

    let has_native_heading = document
        .blocks
        .iter()
        .any(|block| matches!(block.kind, "title" | "heading"));
    let recognized_section_heading_count = document
        .blocks
        .iter()
        .filter(|block| block.kind == "paragraph" && is_recognized_section_heading(&block.text))
        .take(MIN_RECOGNIZED_SECTION_HEADINGS)
        .count();
    if document.status == DocumentStatus::Ok
        && document.blocks.len() >= 3
        && !has_native_heading
        && recognized_section_heading_count < MIN_RECOGNIZED_SECTION_HEADINGS
    {
        add_finding(
            &mut findings,
            Finding {
                rule_id: "cv.structure.no_headings",
                severity: Severity::Warning,
                message: "No heading structure was detected; verify section boundaries and extraction order.".to_owned(),
                evidence: None,
            },
        )?;
    }

    if document.status == DocumentStatus::Ok
        && !document
            .blocks
            .iter()
            .any(|block| contains_email_address(&block.text))
    {
        add_finding(
            &mut findings,
            Finding {
                rule_id: "cv.contact.email_missing",
                severity: Severity::Info,
                message: "No email-like address was found in the extracted text; verify contact details if one should be present.".to_owned(),
                evidence: None,
            },
        )?;
    }

    let mut seen_blocks = HashSet::new();
    for block in &document.blocks {
        if block.text.chars().count() < 20 {
            continue;
        }
        if !seen_blocks.insert(block.text.as_str()) {
            add_finding(
                &mut findings,
                Finding {
                    rule_id: "cv.structure.duplicate_block",
                    severity: Severity::Warning,
                    message:
                        "The same text was extracted more than once; verify document reading order."
                            .to_owned(),
                    evidence: Some(block_evidence(block)),
                },
            )?;
        }
    }

    let spelling_findings = collect_spelling_findings(
        &document.blocks,
        allow_words,
        MAX_FINDINGS.saturating_sub(findings.len()),
    )?;

    Ok(LintReport {
        schema_version: SCHEMA_VERSION,
        status: document.status,
        input: document.input,
        extractor: document.extractor,
        ruleset: ruleset_identity(),
        options: lint_options_identity(allow_words),
        block_count: document.blocks.len(),
        warnings: document.warnings,
        findings,
        spelling_findings,
    })
}

fn is_recognized_section_heading(text: &str) -> bool {
    let normalized = text.trim().trim_end_matches(':').trim().to_lowercase();
    RECOGNIZED_SECTION_HEADINGS.contains(&normalized.as_str())
}

fn collect_spelling_findings(
    blocks: &[TextBlock],
    allow_words: &[String],
    capacity: usize,
) -> Result<Vec<Finding>, ExtractError> {
    let block_texts = blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<Vec<_>>();

    let mut findings = Vec::new();
    for issue in check_blocks(&block_texts, allow_words, capacity.saturating_add(1)) {
        if findings.len() == capacity {
            return Err(ExtractError::TooManyFindings {
                limit: MAX_FINDINGS,
            });
        }
        let block = &blocks[issue.block_index];
        let (rule_id, severity, message) = match issue.classification {
            SpellingClassification::UnknownWord => (
                "cv.spelling.unknown_word",
                Severity::Warning,
                format!(
                    "Possible spelling issue: '{}' was not found in the bundled Swedish or English dictionaries.",
                    issue.term
                ),
            ),
            SpellingClassification::PossibleName => (
                "cv.spelling.possible_name_or_term",
                Severity::Info,
                format!(
                    "Possible name or specialist term: '{}' was not found in the bundled dictionaries; verify it or allowlist it for this run.",
                    issue.term
                ),
            ),
        };
        findings.push(Finding {
            rule_id,
            severity,
            message,
            evidence: Some(FindingEvidence {
                block_id: Some(block.id.clone()),
                source: Some(block.source.clone()),
                block_start_byte: Some(issue.block_start_byte),
                block_end_byte: Some(issue.block_end_byte),
            }),
        });
    }
    Ok(findings)
}

fn block_evidence(block: &TextBlock) -> FindingEvidence {
    FindingEvidence {
        block_id: Some(block.id.clone()),
        source: Some(block.source.clone()),
        block_start_byte: None,
        block_end_byte: None,
    }
}

fn contains_email_address(text: &str) -> bool {
    text.split(|character: char| character.is_whitespace() || "<>()[],;".contains(character))
        .any(|token| {
            let token = token.trim_matches(|character: char| "\"'.:".contains(character));
            let Some((local, domain)) = token.split_once('@') else {
                return false;
            };
            !local.is_empty()
                && domain
                    .split_once('.')
                    .is_some_and(|(name, suffix)| !name.is_empty() && !suffix.is_empty())
        })
}

fn add_finding(findings: &mut Vec<Finding>, finding: Finding) -> Result<(), ExtractError> {
    if findings.len() == MAX_FINDINGS {
        return Err(ExtractError::TooManyFindings {
            limit: MAX_FINDINGS,
        });
    }
    findings.push(finding);
    Ok(())
}

fn is_bidirectional_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
    )
}

fn validate_size(input: &[u8]) -> Result<(), ExtractError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(ExtractError::InputTooLarge {
            actual: input.len(),
            limit: MAX_INPUT_BYTES,
        });
    }
    Ok(())
}

fn validate_allow_words(allow_words: &[String]) -> Result<(), ExtractError> {
    if allow_words.len() > MAX_ALLOW_WORDS {
        return Err(ExtractError::TooManyAllowWords {
            actual: allow_words.len(),
            limit: MAX_ALLOW_WORDS,
        });
    }
    for (index, word) in allow_words.iter().enumerate() {
        if word.len() > MAX_ALLOW_WORD_BYTES {
            return Err(ExtractError::InvalidAllowWord {
                index,
                message: "word exceeds the 360-byte limit",
            });
        }
        if normalize_allow_word(word).is_none() {
            return Err(ExtractError::InvalidAllowWord {
                index,
                message: "expected one alphabetic spelling token without surrounding punctuation, whitespace, or digits",
            });
        }
    }
    Ok(())
}

fn lint_options_identity(allow_words: &[String]) -> LintOptionsIdentity {
    let spelling_allow_words = allow_words
        .iter()
        .filter_map(|word| normalize_allow_word(word))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    LintOptionsIdentity {
        spelling_allow_words,
    }
}

fn input_identity(input: &[u8]) -> InputIdentity {
    let digest = Sha256::digest(input);
    let mut sha256 = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
    }
    InputIdentity {
        sha256,
        size_bytes: input.len(),
    }
}

fn plain_text_extractor() -> ExtractorIdentity {
    ExtractorIdentity {
        name: "plain_text",
        version: env!("CARGO_PKG_VERSION"),
    }
}

fn ruleset_identity() -> RulesetIdentity {
    RulesetIdentity {
        version: RULESET_VERSION,
        spelling: SpellingRulesetIdentity {
            engine: "spellbook",
            engine_version: SPELLBOOK_VERSION,
            dictionary_source: "LibreOffice/dictionaries",
            dictionary_revision: DICTIONARY_SOURCE_COMMIT,
            developer_allowlist_version: DEVELOPER_ALLOWLIST_VERSION,
            matching: "en_US_or_sv_SE",
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct Line<'a> {
    number: usize,
    start_byte: usize,
    end_byte: usize,
    text: &'a str,
}

fn strip_initial_bom<'a>(mut line: Line<'a>) -> Line<'a> {
    if line.number == 1 && line.text.starts_with('\u{feff}') {
        line.start_byte += '\u{feff}'.len_utf8();
        line.text = &line.text['\u{feff}'.len_utf8()..];
    }
    line
}

fn lines_with_offsets(text: &str) -> LinesWithOffsets<'_> {
    LinesWithOffsets {
        text,
        next_start: 0,
        next_line_number: 1,
    }
}

struct LinesWithOffsets<'a> {
    text: &'a str,
    next_start: usize,
    next_line_number: usize,
}

impl<'a> Iterator for LinesWithOffsets<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_start >= self.text.len() {
            return None;
        }

        let start_byte = self.next_start;
        let remaining = &self.text.as_bytes()[start_byte..];
        let content_length = remaining
            .iter()
            .position(|byte| matches!(*byte, b'\r' | b'\n'))
            .unwrap_or(remaining.len());
        let end_byte = start_byte + content_length;

        self.next_start = end_byte;
        if self.next_start < self.text.len() {
            if self.text.as_bytes()[self.next_start] == b'\r'
                && self.text.as_bytes().get(self.next_start + 1) == Some(&b'\n')
            {
                self.next_start += 2;
            } else {
                self.next_start += 1;
            }
        }

        let line = Line {
            number: self.next_line_number,
            start_byte,
            end_byte,
            text: &self.text[start_byte..end_byte],
        };
        self.next_line_number += 1;
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_source(source: &SourceLocator) -> (usize, usize, usize) {
        match source {
            SourceLocator::PlainText {
                line,
                start_byte,
                end_byte,
            } => (*line, *start_byte, *end_byte),
            _ => panic!("expected a plain-text source locator"),
        }
    }

    #[test]
    fn extraction_is_stable_and_preserves_utf8_byte_locations() {
        let input = "Rubrik\r\n\r\nErfarenhet: Göteborg\n".as_bytes();

        let first = extract_plain_text(input).unwrap();
        let second = extract_plain_text(input).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.status, DocumentStatus::Ok);
        assert_eq!(first.blocks.len(), 2);
        assert_eq!(first.blocks[0].id, "block-0001");
        assert_eq!(plain_source(&first.blocks[0].source).0, 1);
        assert_eq!(plain_source(&first.blocks[1].source).0, 3);
        assert_eq!(first.blocks[1].text, "Erfarenhet: Göteborg");
        let (_, start_byte, end_byte) = plain_source(&first.blocks[1].source);
        assert_eq!(
            &input[start_byte..end_byte],
            first.blocks[1].text.as_bytes()
        );
    }

    #[test]
    fn markdown_locations_require_a_complete_conservative_line_match() {
        assert_eq!(markdown_line_content_span("# Alpha", "Alpha"), Some((2, 7)));
        assert_eq!(
            markdown_line_content_span("  - Alpha", "Alpha"),
            Some((4, 9))
        );
        assert_eq!(
            markdown_line_content_span("10. Alpha", "Alpha"),
            Some((4, 9))
        );
        assert_eq!(
            markdown_line_content_span("![Alpha](portrait.png)", "Alpha"),
            None
        );
        assert_eq!(
            markdown_line_content_span("Alpha **middle** Omega", "Alpha middle Omega"),
            None
        );

        let source = "![Alpha](portrait.png)\n\n# Alpha\n";
        let expected = source.rfind("Alpha").unwrap();
        assert_eq!(
            locate_markdown_text(source, "Alpha", 0),
            Some((expected, expected + "Alpha".len()))
        );
    }

    #[test]
    fn section_heading_recognition_is_exact_and_bilingual() {
        assert!(is_recognized_section_heading("SUMMARY:"));
        assert!(is_recognized_section_heading("Arbetslivserfarenhet"));
        assert!(is_recognized_section_heading("Tekniska färdigheter"));
        assert!(!is_recognized_section_heading(
            "Experience building distributed systems"
        ));
        assert!(!is_recognized_section_heading("Education technology"));
    }

    #[test]
    fn cr_only_line_endings_preserve_blocks_and_offsets() {
        let input = b"Name\rRole\rSkills";
        let document = extract_plain_text(input).unwrap();

        assert_eq!(document.blocks.len(), 3);
        assert_eq!(document.blocks[1].text, "Role");
        let (line, start_byte, end_byte) = plain_source(&document.blocks[1].source);
        assert_eq!(line, 2);
        assert_eq!(start_byte, 5);
        assert_eq!(end_byte, 9);
        assert_eq!(&input[start_byte..end_byte], b"Role");
    }

    #[test]
    fn initial_bom_is_removed_from_text_and_reported() {
        let input = b"\xef\xbb\xbfName\n";
        let document = extract_plain_text(input).unwrap();
        let report = lint_plain_text(input).unwrap();

        assert_eq!(document.blocks[0].text, "Name");
        assert_eq!(plain_source(&document.blocks[0].source).1, 3);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.rule_id == "text.utf8_bom")
            .unwrap();
        let evidence = finding.evidence.as_ref().unwrap();
        assert_eq!(plain_source(evidence.source.as_ref().unwrap()).1, 0);
        assert_eq!(plain_source(evidence.source.as_ref().unwrap()).2, 3);
    }

    #[test]
    fn unicode_format_and_bidirectional_controls_are_reported() {
        let report = lint_plain_text("Rust\u{200b}\u{202e}\n".as_bytes()).unwrap();
        let rule_ids = report
            .findings
            .iter()
            .map(|finding| finding.rule_id)
            .collect::<Vec<_>>();

        assert_eq!(
            rule_ids,
            vec!["text.format_character", "text.bidirectional_control"]
        );
    }

    #[test]
    fn trailing_whitespace_evidence_covers_the_complete_run() {
        let report = lint_plain_text(b"Role \t \n").unwrap();
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.rule_id == "text.trailing_whitespace")
            .unwrap();
        let evidence = finding.evidence.as_ref().unwrap();

        assert_eq!(plain_source(evidence.source.as_ref().unwrap()).1, 4);
        assert_eq!(plain_source(evidence.source.as_ref().unwrap()).2, 7);
    }

    #[test]
    fn lint_handles_many_lines_in_one_pass() {
        let input = "entry\n".repeat(MAX_TEXT_LINES);
        let report = lint_plain_text(input.as_bytes()).unwrap();

        assert_eq!(report.block_count, MAX_TEXT_LINES);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn excessive_line_and_finding_counts_are_rejected() {
        let too_many_lines = "entry\n".repeat(MAX_TEXT_LINES + 1);
        assert_eq!(
            extract_plain_text(too_many_lines.as_bytes()).unwrap_err(),
            ExtractError::TooManyLines {
                limit: MAX_TEXT_LINES
            }
        );

        let too_many_findings = "\t".repeat(MAX_FINDINGS + 1);
        assert_eq!(
            lint_plain_text(too_many_findings.as_bytes()).unwrap_err(),
            ExtractError::TooManyFindings {
                limit: MAX_FINDINGS
            }
        );
    }

    #[test]
    fn empty_text_has_an_explicit_status_and_finding() {
        let report = lint_plain_text(b"  \n\t\n").unwrap();

        assert_eq!(report.status, DocumentStatus::NoReadableText);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == "text.empty")
        );
    }

    #[test]
    fn lint_reports_deterministic_byte_derived_findings() {
        let report = lint_plain_text(b"Name\t \nRole\0\n").unwrap();
        let rule_ids = report
            .findings
            .iter()
            .map(|finding| finding.rule_id)
            .collect::<Vec<_>>();

        assert_eq!(
            rule_ids,
            vec![
                "text.trailing_whitespace",
                "text.tab_character",
                "text.nul_character"
            ]
        );
        assert_eq!(
            plain_source(
                report.findings[1]
                    .evidence
                    .as_ref()
                    .unwrap()
                    .source
                    .as_ref()
                    .unwrap()
            )
            .0,
            1
        );
        assert_eq!(
            plain_source(
                report.findings[2]
                    .evidence
                    .as_ref()
                    .unwrap()
                    .source
                    .as_ref()
                    .unwrap()
            )
            .1,
            11
        );
    }

    #[test]
    fn invalid_utf8_is_rejected_with_a_byte_offset() {
        let error = extract_plain_text(&[b'A', 0xff]).unwrap_err();
        assert_eq!(error, ExtractError::InvalidUtf8 { valid_up_to: 1 });
    }

    #[test]
    fn oversized_input_is_rejected() {
        let input = vec![b'x'; MAX_INPUT_BYTES + 1];
        let error = extract_plain_text(&input).unwrap_err();
        assert_eq!(
            error,
            ExtractError::InputTooLarge {
                actual: MAX_INPUT_BYTES + 1,
                limit: MAX_INPUT_BYTES
            }
        );
    }

    #[test]
    fn selected_input_must_be_a_regular_file() {
        let directory = tempfile::tempdir().unwrap();
        let error = read_selected_file(directory.path()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn spelling_allow_words_are_bounded_and_single_term() {
        let too_many = vec!["term".to_owned(); MAX_ALLOW_WORDS + 1];
        assert_eq!(
            validate_allow_words(&too_many).unwrap_err(),
            ExtractError::TooManyAllowWords {
                actual: MAX_ALLOW_WORDS + 1,
                limit: MAX_ALLOW_WORDS,
            }
        );
        assert!(matches!(
            validate_allow_words(&["two words".to_owned()]).unwrap_err(),
            ExtractError::InvalidAllowWord { .. }
        ));
        assert!(matches!(
            validate_allow_words(&["term,".to_owned()]).unwrap_err(),
            ExtractError::InvalidAllowWord { .. }
        ));
    }

    #[test]
    fn spelling_findings_respect_the_global_finding_boundary() {
        let filler = Finding {
            rule_id: "test.filler",
            severity: Severity::Info,
            message: "filler".to_owned(),
            evidence: None,
        };
        let block = TextBlock {
            id: "block-0001".to_owned(),
            kind: "line",
            text: "quuxzorp".to_owned(),
            source: SourceLocator::PlainText {
                line: 1,
                start_byte: 0,
                end_byte: 8,
            },
            provenance: "native_text",
        };

        let at_boundary = vec![filler.clone(); MAX_FINDINGS - 1];
        let spelling = collect_spelling_findings(
            std::slice::from_ref(&block),
            &[],
            MAX_FINDINGS - at_boundary.len(),
        )
        .unwrap();
        assert_eq!(at_boundary.len() + spelling.len(), MAX_FINDINGS);

        let over_boundary = vec![filler; MAX_FINDINGS];
        assert_eq!(
            collect_spelling_findings(&[block], &[], MAX_FINDINGS - over_boundary.len())
                .unwrap_err(),
            ExtractError::TooManyFindings {
                limit: MAX_FINDINGS
            }
        );
    }
}
