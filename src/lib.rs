use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::{self, Write as _};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_TEXT_LINES: usize = 100_000;
pub const MAX_FINDINGS: usize = 10_000;
pub const SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    InputTooLarge { actual: usize, limit: usize },
    InvalidUtf8 { valid_up_to: usize },
    TooManyLines { limit: usize },
    TooManyFindings { limit: usize },
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
            Self::TooManyFindings { limit } => {
                write!(formatter, "lint exceeds the {limit}-finding limit")
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
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Ok,
    NoReadableText,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceLocator {
    pub line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TextBlock {
    pub id: String,
    pub kind: &'static str,
    pub text: String,
    pub source: SourceLocator,
    pub provenance: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub schema_version: &'static str,
    pub status: DocumentStatus,
    pub input: InputIdentity,
    pub extractor: ExtractorIdentity,
    pub blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FindingEvidence {
    pub block_id: Option<String>,
    pub line: Option<usize>,
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub message: &'static str,
    pub evidence: Option<FindingEvidence>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LintReport {
    pub schema_version: &'static str,
    pub status: DocumentStatus,
    pub input: InputIdentity,
    pub extractor: ExtractorIdentity,
    pub block_count: usize,
    pub findings: Vec<Finding>,
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
            source: SourceLocator {
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
    })
}

pub fn lint_plain_text(input: &[u8]) -> Result<LintReport, ExtractError> {
    let document = extract_plain_text(input)?;
    let text = std::str::from_utf8(input).expect("extract_plain_text already validated UTF-8");
    let mut findings = Vec::new();

    if document.status == DocumentStatus::NoReadableText {
        add_finding(
            &mut findings,
            Finding {
                rule_id: "text.empty",
                severity: Severity::Error,
                message: "The document contains no readable text.",
                evidence: None,
            },
        )?;
    }

    let mut blocks = document.blocks.iter().peekable();
    for line in lines_with_offsets(text) {
        let block_id = if blocks
            .peek()
            .is_some_and(|block| block.source.line == line.number)
        {
            blocks.next().map(|block| block.id.clone())
        } else {
            None
        };

        let without_trailing_whitespace = line.text.trim_end_matches(char::is_whitespace);
        if without_trailing_whitespace.len() != line.text.len() {
            add_finding(
                &mut findings,
                Finding {
                    rule_id: "text.trailing_whitespace",
                    severity: Severity::Info,
                    message: "The line has trailing whitespace.",
                    evidence: Some(FindingEvidence {
                        block_id: block_id.clone(),
                        line: Some(line.number),
                        start_byte: Some(line.start_byte + without_trailing_whitespace.len()),
                        end_byte: Some(line.end_byte),
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
                        message,
                        evidence: Some(FindingEvidence {
                            block_id: block_id.clone(),
                            line: Some(line.number),
                            start_byte: Some(absolute_offset),
                            end_byte: Some(absolute_offset + character.len_utf8()),
                        }),
                    },
                )?;
            }
        }
    }

    Ok(LintReport {
        schema_version: SCHEMA_VERSION,
        status: document.status,
        input: document.input,
        extractor: document.extractor,
        block_count: document.blocks.len(),
        findings,
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

    #[test]
    fn extraction_is_stable_and_preserves_utf8_byte_locations() {
        let input = "Rubrik\r\n\r\nErfarenhet: Göteborg\n".as_bytes();

        let first = extract_plain_text(input).unwrap();
        let second = extract_plain_text(input).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.status, DocumentStatus::Ok);
        assert_eq!(first.blocks.len(), 2);
        assert_eq!(first.blocks[0].id, "block-0001");
        assert_eq!(first.blocks[0].source.line, 1);
        assert_eq!(first.blocks[1].source.line, 3);
        assert_eq!(first.blocks[1].text, "Erfarenhet: Göteborg");
        assert_eq!(
            &input[first.blocks[1].source.start_byte..first.blocks[1].source.end_byte],
            first.blocks[1].text.as_bytes()
        );
    }

    #[test]
    fn cr_only_line_endings_preserve_blocks_and_offsets() {
        let input = b"Name\rRole\rSkills";
        let document = extract_plain_text(input).unwrap();

        assert_eq!(document.blocks.len(), 3);
        assert_eq!(document.blocks[1].text, "Role");
        assert_eq!(document.blocks[1].source.line, 2);
        assert_eq!(document.blocks[1].source.start_byte, 5);
        assert_eq!(document.blocks[1].source.end_byte, 9);
        assert_eq!(
            &input[document.blocks[1].source.start_byte..document.blocks[1].source.end_byte],
            b"Role"
        );
    }

    #[test]
    fn initial_bom_is_removed_from_text_and_reported() {
        let input = b"\xef\xbb\xbfName\n";
        let document = extract_plain_text(input).unwrap();
        let report = lint_plain_text(input).unwrap();

        assert_eq!(document.blocks[0].text, "Name");
        assert_eq!(document.blocks[0].source.start_byte, 3);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.rule_id == "text.utf8_bom")
            .unwrap();
        let evidence = finding.evidence.as_ref().unwrap();
        assert_eq!(evidence.start_byte, Some(0));
        assert_eq!(evidence.end_byte, Some(3));
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

        assert_eq!(evidence.start_byte, Some(4));
        assert_eq!(evidence.end_byte, Some(7));
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
        assert_eq!(report.findings[1].evidence.as_ref().unwrap().line, Some(1));
        assert_eq!(
            report.findings[2].evidence.as_ref().unwrap().start_byte,
            Some(11)
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
}
