mod common;

use cv_linter::{
    DocumentStatus, ExtractError, MAX_DOCUMENT_BLOCKS, Severity, SourceLocator, extract_document,
    lint_document, lint_document_with_allow_words,
};
use std::path::Path;

#[tokio::test]
async fn markdown_extraction_is_stable_and_byte_located() {
    let input = include_bytes!("fixtures/swedish-cv.md");

    let first = extract_document(Path::new("synthetic-cv.md"), input)
        .await
        .unwrap();
    let second = extract_document(Path::new("synthetic-cv.md"), input)
        .await
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.status, DocumentStatus::Ok);
    assert_eq!(first.extractor.name, "xberg");
    assert_eq!(first.extractor.version, "1.1.5");
    assert_eq!(first.blocks[0].text, "Alex Andersson");
    assert_eq!(first.blocks[0].kind, "heading");
    match first.blocks[0].source {
        SourceLocator::Markdown {
            line,
            start_byte,
            end_byte,
            ordered_block,
        } => {
            assert_eq!(line, Some(1));
            assert_eq!(ordered_block, 1);
            let source = std::str::from_utf8(input).unwrap();
            let range = start_byte.unwrap()..end_byte.unwrap();
            assert!(source[range].contains("Alex Andersson"));
        }
        ref source => panic!("expected Markdown locator, got {source:?}"),
    }
}

#[tokio::test]
async fn transformed_markdown_never_receives_a_guessed_source_range() {
    let input = b"![Alpha](portrait.png)\n\nAlpha **middle** Omega\n\n# Skills\n";

    let document = extract_document(Path::new("styled-cv.md"), input)
        .await
        .unwrap();
    let block = document
        .blocks
        .iter()
        .find(|block| block.text.contains("Alpha middle Omega"))
        .unwrap();

    assert!(matches!(
        block.source,
        SourceLocator::Markdown {
            line: None,
            start_byte: None,
            end_byte: None,
            ..
        }
    ));

    let later_heading = document
        .blocks
        .iter()
        .find(|block| block.text == "Skills")
        .unwrap();
    assert!(matches!(
        later_heading.source,
        SourceLocator::Markdown { line: Some(5), .. }
    ));
}

#[tokio::test]
async fn pdf_extraction_returns_page_located_native_text() {
    let input = common::text_pdf(&[
        "Alex Andersson",
        "alex.andersson@example.com",
        "Profil",
        "Senior Rustutvecklare i Stockholm",
        "Erfarenhet",
        "Byggde lokala analysverktyg i Rust",
    ]);

    let document = extract_document(Path::new("synthetic-cv.pdf"), &input)
        .await
        .unwrap();

    assert_eq!(document.status, DocumentStatus::Ok);
    assert!(
        document
            .blocks
            .iter()
            .any(|block| block.text.contains("Alex Andersson"))
    );
    assert!(
        document
            .blocks
            .iter()
            .all(|block| { matches!(block.source, SourceLocator::Pdf { page: Some(1), .. }) })
    );
}

#[tokio::test]
async fn recognizable_pdf_section_labels_satisfy_structure_lint() {
    let input = common::text_pdf(&[
        "Alex Andersson",
        "alex.andersson@example.com",
        "Summary",
        "Software engineering manager in Stockholm.",
        "Experience",
        "Led a cross-functional product team.",
    ]);
    let document = extract_document(Path::new("sectioned-cv.pdf"), &input)
        .await
        .unwrap();
    assert!(
        document
            .blocks
            .iter()
            .all(|block| block.kind == "paragraph")
    );

    let report = lint_document(Path::new("sectioned-cv.pdf"), &input)
        .await
        .unwrap();
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.rule_id != "cv.structure.no_headings")
    );
}

#[tokio::test]
async fn chrome_pdf_with_flattened_headings_has_recognizable_section_structure() {
    let input = include_bytes!("fixtures/chrome-tagged-cv.pdf");
    let document = extract_document(Path::new("chrome-tagged-cv.pdf"), input)
        .await
        .unwrap();

    assert_eq!(document.status, DocumentStatus::Ok);
    assert!(document.warnings.is_empty());
    for section in [
        "Summary",
        "Core Competencies",
        "Experience",
        "Technical Skills",
        "Projects",
        "Education",
        "Languages",
        "Outside Work",
    ] {
        assert!(document.blocks.iter().any(|block| block.text == section));
    }

    let report = lint_document(Path::new("chrome-tagged-cv.pdf"), input)
        .await
        .unwrap();
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.rule_id != "cv.structure.no_headings")
    );
}

#[tokio::test]
async fn docx_extraction_preserves_swedish_text_and_element_locations() {
    let input = common::docx(&[
        "Alex Andersson",
        "alex.andersson@example.com",
        "Profil",
        "Rustutvecklare i Göteborg med erfarenhet av lokala verktyg.",
    ]);

    let document = extract_document(Path::new("synthetic-cv.docx"), &input)
        .await
        .unwrap();

    assert_eq!(document.status, DocumentStatus::Ok);
    assert!(
        document
            .blocks
            .iter()
            .any(|block| block.text.contains("Göteborg"))
    );
    assert!(
        document
            .blocks
            .iter()
            .all(|block| matches!(block.source, SourceLocator::Docx { .. }))
    );
}

#[tokio::test]
async fn blank_pdf_reports_the_no_native_text_limitation() {
    let input = common::blank_pdf();
    let document = extract_document(Path::new("scan.pdf"), &input)
        .await
        .unwrap();

    assert_eq!(document.status, DocumentStatus::NoReadableText);
    assert!(document.blocks.is_empty());
    assert!(
        document
            .warnings
            .iter()
            .any(|warning| warning.code == "extraction.no_native_text")
    );

    let report = lint_document(Path::new("scan.pdf"), &input).await.unwrap();
    assert!(
        report.findings.iter().any(|finding| {
            finding.rule_id == "text.empty" && finding.severity == Severity::Error
        })
    );
}

#[tokio::test]
async fn unsupported_and_malformed_documents_fail_explicitly() {
    let unsupported = extract_document(Path::new("cv.rtf"), b"plain text")
        .await
        .unwrap_err();
    assert!(matches!(
        unsupported,
        ExtractError::UnsupportedFormat { .. }
    ));

    let malformed = extract_document(Path::new("cv.pdf"), b"not a PDF")
        .await
        .unwrap_err();
    assert!(matches!(
        malformed,
        ExtractError::ExtractionFailed { format: "PDF", .. }
    ));

    let malformed_docx = extract_document(Path::new("cv.docx"), b"not a DOCX")
        .await
        .unwrap_err();
    assert!(matches!(
        malformed_docx,
        ExtractError::ExtractionFailed { format: "DOCX", .. }
    ));

    let encrypted = extract_document(Path::new("protected.pdf"), &common::encrypted_pdf())
        .await
        .unwrap_err();
    assert!(matches!(
        &encrypted,
        ExtractError::ExtractionFailed { format: "PDF", .. }
    ));
    assert!(encrypted.to_string().to_lowercase().contains("password"));
}

#[tokio::test]
async fn excessive_markdown_is_rejected_before_native_extraction() {
    let input = "# x\n".repeat(MAX_DOCUMENT_BLOCKS + 1);

    let error = extract_document(Path::new("oversized-cv.md"), input.as_bytes())
        .await
        .unwrap_err();

    assert_eq!(
        error,
        ExtractError::TooManyLines {
            limit: MAX_DOCUMENT_BLOCKS
        }
    );
}

#[tokio::test]
async fn lint_adds_deterministic_cv_structure_findings() {
    let input = b"First paragraph\n\nSecond paragraph\n\nThird paragraph";
    let report = lint_document(Path::new("cv.md"), input).await.unwrap();
    let rule_ids = report
        .findings
        .iter()
        .map(|finding| finding.rule_id)
        .collect::<Vec<_>>();

    assert!(rule_ids.contains(&"cv.structure.no_headings"));
    assert!(rule_ids.contains(&"cv.contact.email_missing"));
    assert_eq!(report.ruleset.version, "0.2.2");
    assert_eq!(report.ruleset.spelling.engine, "spellbook");
    assert_eq!(report.ruleset.spelling.engine_version, "0.4.2");
}

#[tokio::test]
async fn bilingual_spelling_is_located_deduplicated_and_allowlisted_per_run() {
    let input = b"# Profil\n\nalex@example.com\n\nerfarenhett experiance Franglemist erfarenhett\n";
    let report = lint_document(Path::new("typos.md"), input).await.unwrap();
    let spelling = &report.spelling_findings;

    assert!(
        report
            .findings
            .iter()
            .all(|finding| !finding.rule_id.starts_with("cv.spelling."))
    );
    assert_eq!(spelling.len(), 3);
    assert_eq!(spelling[0].severity, Severity::Warning);
    assert!(spelling[0].message.contains("erfarenhett"));
    assert!(spelling[1].message.contains("experiance"));
    assert_eq!(spelling[2].severity, Severity::Info);
    assert!(spelling[2].message.contains("Franglemist"));
    let evidence = spelling[0].evidence.as_ref().unwrap();
    assert_eq!(evidence.block_start_byte, Some(0));
    assert_eq!(evidence.block_end_byte, Some("erfarenhett".len()));

    let allow_words = vec![
        "Franglemist".to_owned(),
        "experiance".to_owned(),
        "ERFARENHETT".to_owned(),
        "franglemist".to_owned(),
    ];
    let allowed = lint_document_with_allow_words(Path::new("typos.md"), input, &allow_words)
        .await
        .unwrap();
    assert_eq!(
        allowed.options.spelling_allow_words,
        vec!["erfarenhett", "experiance", "franglemist"]
    );
    assert!(allowed.spelling_findings.is_empty());
}
