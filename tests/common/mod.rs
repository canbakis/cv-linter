//! Small, deterministic document fixtures shared by integration tests.

#![allow(dead_code)]

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Build a one-page PDF whose body contains one text line per input item.
///
/// The PDF is deliberately assembled here instead of through a PDF crate so
/// that tests do not depend on another parser or renderer. Non-ASCII bytes are
/// represented with PDF octal escapes, keeping the enclosing PDF ASCII.
pub fn text_pdf(lines: &[&str]) -> Vec<u8> {
    let mut content = String::from("BT /F1 12 Tf 72 720 Td ");
    for (index, line) in lines.iter().enumerate() {
        if index != 0 {
            content.push_str("0 -16 Td ");
        }
        content.push('(');
        content.push_str(&pdf_literal(line));
        content.push_str(") Tj ");
    }
    content.push_str("ET\n");

    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{}endstream", content.len(), content),
    ];

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len() + 1);
    offsets.push(0);
    for (number, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", number + 1, object).as_bytes());
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    pdf
}

/// Build an empty, but structurally valid, one-page PDF.
pub fn blank_pdf() -> Vec<u8> {
    text_pdf(&[])
}

/// A synthetic AES-256 PDF that requires the password used by Xberg's own
/// password-required fixture. The product does not accept PDF passwords.
pub fn encrypted_pdf() -> Vec<u8> {
    use base64::Engine as _;

    const PDF: &str = concat!(
        "JVBERi0xLjcKJb/3ov4KMSAwIG9iago8PCAvRXh0ZW5zaW9ucyA8PCAvQURCRSA8PCAvQmFzZVZlcnNpb24gLzEuNyAv",
        "RXh0ZW5zaW9uTGV2ZWwgOCA+PiA+PiAvUGFnZXMgMiAwIFIgL1R5cGUgL0NhdGFsb2cgPj4KZW5kb2JqCjIgMCBvYmoK",
        "PDwgL0NvdW50IDMgL0tpZHMgWyAzIDAgUiA0IDAgUiA1IDAgUiBdIC9UeXBlIC9QYWdlcyA+PgplbmRvYmoKMyAwIG9i",
        "ago8PCAvQ29udGVudHMgNiAwIFIgL01lZGlhQm94IFsgMCAwIDIwMCAyMDAgXSAvUGFyZW50IDIgMCBSIC9SZXNvdXJj",
        "ZXMgPDwgL0ZvbnQgPDwgL0YxIDcgMCBSID4+ID4+IC9UeXBlIC9QYWdlID4+CmVuZG9iago0IDAgb2JqCjw8IC9Db250",
        "ZW50cyA4IDAgUiAvTWVkaWFCb3ggWyAwIDAgMjAwIDIwMCBdIC9QYXJlbnQgMiAwIFIgL1Jlc291cmNlcyA8PCAvRm9u",
        "dCA8PCAvRjEgNyAwIFIgPj4gPj4gL1R5cGUgL1BhZ2UgPj4KZW5kb2JqCjUgMCBvYmoKPDwgL0NvbnRlbnRzIDkgMCBS",
        "IC9NZWRpYUJveCBbIDAgMCAyMDAgMjAwIF0gL1BhcmVudCAyIDAgUiAvUmVzb3VyY2VzIDw8IC9Gb250IDw8IC9GMSA3",
        "IDAgUiA+PiA+PiAvVHlwZSAvUGFnZSA+PgplbmRvYmoKNiAwIG9iago8PCAvRmlsdGVyIC9GbGF0ZURlY29kZSAvTGVu",
        "Z3RoIDgwID4+CnN0cmVhbQq/uprLnpDr1L1WYuji7aoo8FC/AmNCG9HIRQLXttVpBBc7g+Wbs7gcxVkaZNEjLHKAT0E4",
        "zK38JxdAJjfL7XkpkCbptuaCZCGvvfQ0dyrxlWVuZHN0cmVhbQplbmRvYmoKNyAwIG9iago8PCAvQmFzZUZvbnQgL0hl",
        "bHZldGljYSAvU3VidHlwZSAvVHlwZTEgL1R5cGUgL0ZvbnQgPj4KZW5kb2JqCjggMCBvYmoKPDwgL0ZpbHRlciAvRmxh",
        "dGVEZWNvZGUgL0xlbmd0aCA4MCA+PgpzdHJlYW0K4JE0iPOQNpJRjCvrfuVuZ8G+7e3bbkuci4qETQEnA0RfeOqEZe5q",
        "L81EqqU/h7+KsfI+uoIT6tBO4uAdnj/i0054F0Q6VoQWb0PkpOiMLy1lbmRzdHJlYW0KZW5kb2JqCjkgMCBvYmoKPDwg",
        "L0ZpbHRlciAvRmxhdGVEZWNvZGUgL0xlbmd0aCA4MCA+PgpzdHJlYW0KKqDRR85TmYv59t1N7OdBCttbZfgU8lsnHRFY",
        "MueyVRSwbBDvnujXiRLztgcPFiS4XexkTW/ikEXnPHq9uFq1VDpPVQNpaRQlZKf9xlAUveVlbmRzdHJlYW0KZW5kb2Jq",
        "CjEwIDAgb2JqCjw8IC9DRiA8PCAvU3RkQ0YgPDwgL0F1dGhFdmVudCAvRG9jT3BlbiAvQ0ZNIC9BRVNWMyAvTGVuZ3Ro",
        "IDMyID4+ID4+IC9GaWx0ZXIgL1N0YW5kYXJkIC9MZW5ndGggMjU2IC9PIDw0ZmQzOTFhNjY0MzdlZjFkYzEwNDMxOGVj",
        "OTZhNDY1OTM5OWNhMzY2YTY1MTBmZjhkNTI1N2FjMmJiOGRmMGNkMGRiZjkzYTc4MDgzYTkzOGRhYzBlMjcwMzliMTMw",
        "ZmM+IC9PRSA8NDkwZTZlNzI0OGM2NmZkMjZjOTdjMzE3MWVlNjVlNjc0Yjk1YWM2ZWI1MTI1ZjVlZmVkMDBmOTI3NmQw",
        "Y2YxMj4gL1AgLTQgL1Blcm1zIDw1NmI5YWRkMWJiMWI2YjUwZGYyMjg3NWM1Njc5YTJmMz4gL1IgNiAvU3RtRiAvU3Rk",
        "Q0YgL1N0ckYgL1N0ZENGIC9VIDw0NTNlNjI2Y2JmZTI1MGE5N2FmNDllZWEzMmYwOWFmOWMyZDk4Y2MzMWM2ZGQ1NDdj",
        "NjkwYzU1NTMwZDhjN2ViYjA2MzdlN2JjZmZmMDdkZThmYjc2NmFmNWQ4ZDM1NWY+IC9VRSA8MTA0YWEyZjAzNDBiMTEw",
        "ZGEzZTFmMGQ0ODQ3NmQyOWMyZTExNmYxOTRlZTBjMjFmYWFiMzkzNDFiZmU2NjMyZT4gL1YgNSA+PgplbmRvYmoKeHJl",
        "ZgowIDExCjAwMDAwMDAwMDAgNjU1MzUgZiAKMDAwMDAwMDAxNSAwMDAwMCBuIAowMDAwMDAwMTMwIDAwMDAwIG4gCjAw",
        "MDAwMDAyMDEgMDAwMDAgbiAKMDAwMDAwMDMyOSAwMDAwMCBuIAowMDAwMDAwNDU3IDAwMDAwIG4gCjAwMDAwMDA1ODUg",
        "MDAwMDAgbiAKMDAwMDAwMDczNSAwMDAwMCBuIAowMDAwMDAwODA1IDAwMDAwIG4gCjAwMDAwMDA5NTUgMDAwMDAgbiAK",
        "MDAwMDAwMTEwNSAwMDAwMCBuIAp0cmFpbGVyIDw8IC9Sb290IDEgMCBSIC9TaXplIDExIC9JRCBbPGU4ODhiODFiZjdj",
        "YzRhODE3NzQ3YmM4ZDZlZTgxYzgxPjxmYzI0MzUwMTY1NzA4NjdhNDYzNTZiYTFiYWE5ZGVhND5dIC9FbmNyeXB0IDEw",
        "IDAgUiA+PgpzdGFydHhyZWYKMTY1MwolJUVPRgo="
    );

    base64::engine::general_purpose::STANDARD
        .decode(PDF)
        .expect("embedded encrypted PDF fixture")
}

/// Build a minimal DOCX containing one paragraph per input item.
pub fn docx(paragraphs: &[&str]) -> Vec<u8> {
    let body = paragraphs
        .iter()
        .map(|paragraph| {
            format!(
                "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                xml_escape(paragraph)
            )
        })
        .collect::<String>();
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{body}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr></w:body></w:document>"
    );

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, contents) in [
        (
            "[Content_Types].xml",
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>",
        ),
        (
            "_rels/.rels",
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>",
        ),
        ("word/document.xml", document.as_str()),
    ] {
        writer
            .start_file(name, options)
            .expect("in-memory DOCX entry");
        writer
            .write_all(contents.as_bytes())
            .expect("in-memory DOCX contents");
    }
    writer.finish().expect("finish in-memory DOCX").into_inner()
}

fn pdf_literal(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'(' => "\\(".to_owned(),
            b')' => "\\)".to_owned(),
            b'\\' => "\\\\".to_owned(),
            32..=126 => (byte as char).to_string(),
            _ => format!("\\{byte:03o}"),
        })
        .collect()
}

fn xml_escape(value: &str) -> String {
    value.chars().fold(String::new(), |mut escaped, character| {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            character => escaped.push(character),
        }
        escaped
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn pdf_has_consistent_xref() {
        let pdf = text_pdf(&["A (test)", "B\\C"]);
        assert!(pdf.windows(5).any(|window| window == b"xref\n"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn docx_contains_escaped_utf8_text() {
        let mut archive = zip::ZipArchive::new(Cursor::new(docx(&["Förmåga & <test>"]))).unwrap();
        let mut document = String::new();
        archive
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut document)
            .unwrap();
        assert!(document.contains("Förmåga &amp; &lt;test&gt;"));
    }
}
