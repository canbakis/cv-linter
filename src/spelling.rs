use serde::Serialize;
use spellbook::{Dictionary, MAX_WORD_LEN};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::LazyLock;
use unicode_general_category::{GeneralCategory, get_general_category};

pub const SPELLBOOK_VERSION: &str = "0.4.2";
pub const DICTIONARY_SOURCE_COMMIT: &str = "32b006a2c22a4ac7e8ed3f03346f7b3d85a970a4";
pub const DEVELOPER_ALLOWLIST_VERSION: &str = "0.1.0";

const EN_US_AFF: &str = include_str!("../dictionaries/en_US/en_US.aff");
const EN_US_DIC: &str = include_str!("../dictionaries/en_US/en_US.dic");
const SV_SE_AFF: &str = include_str!("../dictionaries/sv_SE/sv_SE.aff");
const SV_SE_DIC: &str = include_str!("../dictionaries/sv_SE/sv_SE.dic");

// Keep this deliberately small. It covers common developer terms that occur in
// CVs but are inconsistently represented in the bundled general dictionaries.
const DEVELOPER_ALLOWLIST: &[&str] = &[
    "api",
    "apis",
    "aws",
    "backend",
    "cloudflare",
    "codex",
    "devops",
    "docker",
    "frontend",
    "fullstack",
    "gcp",
    "github",
    "gitlab",
    "graphql",
    "javascript",
    "js",
    "kubernetes",
    "linkedin",
    "microservice",
    "microservices",
    "nosql",
    "postgresql",
    "react",
    "rest",
    "rust",
    "sql",
    "typescript",
    "websocket",
    "websockets",
    "xberg",
];

static BUNDLED_DICTIONARIES: LazyLock<BilingualDictionaries> = LazyLock::new(|| {
    BilingualDictionaries::new().expect("bundled spelling dictionaries must parse")
});

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpellingClassification {
    UnknownWord,
    PossibleName,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SpellingIssue {
    pub block_index: usize,
    pub term: String,
    pub normalized_term: String,
    pub block_start_byte: usize,
    pub block_end_byte: usize,
    pub classification: SpellingClassification,
}

/// Checks text blocks in source order using the bundled English and Swedish
/// dictionaries. Only the first occurrence of each case-folded unknown term is
/// returned. User allow words are compared case-insensitively and are not
/// persisted.
#[must_use]
pub fn check_blocks(
    blocks: &[&str],
    user_allow_words: &[String],
    max_issues: usize,
) -> Vec<SpellingIssue> {
    BUNDLED_DICTIONARIES.check_blocks(blocks, user_allow_words, max_issues)
}

struct BilingualDictionaries {
    en_us: Dictionary,
    sv_se: Dictionary,
}

impl BilingualDictionaries {
    fn new() -> Result<Self, spellbook::ParseDictionaryError> {
        Ok(Self {
            en_us: Dictionary::new(EN_US_AFF, EN_US_DIC)?,
            sv_se: Dictionary::new(SV_SE_AFF, SV_SE_DIC)?,
        })
    }

    fn check_blocks(
        &self,
        blocks: &[&str],
        user_allow_words: &[String],
        max_issues: usize,
    ) -> Vec<SpellingIssue> {
        if max_issues == 0 {
            return Vec::new();
        }

        let user_allow_words: HashSet<String> = user_allow_words
            .iter()
            .map(|word| normalize(word))
            .filter(|word| !word.is_empty())
            .collect();
        let mut seen_terms = HashSet::new();
        let mut issues = Vec::new();

        for (block_index, block) in blocks.iter().enumerate() {
            for chunk in chunks(block) {
                if looks_like_email_or_url(chunk.text) {
                    continue;
                }

                for token in tokens(chunk) {
                    if token.text.len() > MAX_WORD_LEN
                        || token.text.chars().any(char::is_numeric)
                        || !token.text.chars().any(char::is_alphabetic)
                    {
                        continue;
                    }

                    let normalized = normalize_for_check(token.text);
                    if seen_terms.contains(normalized.as_ref()) {
                        continue;
                    }
                    let accepted = DEVELOPER_ALLOWLIST.contains(&normalized.as_ref())
                        || user_allow_words.contains(normalized.as_ref())
                        || self.en_us.check(token.text)
                        || self.sv_se.check(token.text)
                        || (normalized.as_ref() != token.text
                            && (self.en_us.check(normalized.as_ref())
                                || self.sv_se.check(normalized.as_ref())));
                    let normalized = normalized.into_owned();
                    seen_terms.insert(normalized.clone());
                    if accepted {
                        continue;
                    }

                    let classification = if token
                        .text
                        .chars()
                        .find(|character| character.is_alphabetic())
                        .is_some_and(char::is_uppercase)
                    {
                        SpellingClassification::PossibleName
                    } else {
                        SpellingClassification::UnknownWord
                    };

                    issues.push(SpellingIssue {
                        block_index,
                        term: token.text.to_owned(),
                        normalized_term: normalized,
                        block_start_byte: token.start_byte,
                        block_end_byte: token.end_byte,
                        classification,
                    });
                    if issues.len() == max_issues {
                        return issues;
                    }
                }
            }
        }

        issues
    }
}

#[derive(Clone, Copy)]
struct TextSpan<'a> {
    text: &'a str,
    start_byte: usize,
    end_byte: usize,
}

fn chunks(text: &str) -> Chunks<'_> {
    Chunks {
        remaining: text,
        offset: 0,
    }
}

struct Chunks<'a> {
    remaining: &'a str,
    offset: usize,
}

impl<'a> Iterator for Chunks<'a> {
    type Item = TextSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let leading_bytes = self
            .remaining
            .find(|character: char| !character.is_whitespace())?;
        self.remaining = &self.remaining[leading_bytes..];
        self.offset += leading_bytes;

        let length = self
            .remaining
            .find(char::is_whitespace)
            .unwrap_or(self.remaining.len());
        let span = TextSpan {
            text: &self.remaining[..length],
            start_byte: self.offset,
            end_byte: self.offset + length,
        };
        self.remaining = &self.remaining[length..];
        self.offset += length;
        Some(span)
    }
}

fn tokens(chunk: TextSpan<'_>) -> Tokens<'_> {
    Tokens { chunk, cursor: 0 }
}

struct Tokens<'a> {
    chunk: TextSpan<'a>,
    cursor: usize,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = TextSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let relative_start = self.chunk.text[self.cursor..]
            .find(is_word_character)
            .map(|start| self.cursor + start)?;
        let mut end = relative_start;
        let mut characters = self.chunk.text[relative_start..].char_indices().peekable();

        while let Some((relative_byte, character)) = characters.next() {
            let byte = relative_start + relative_byte;
            let connector_between_word_characters = is_connector(character)
                && characters
                    .peek()
                    .is_some_and(|(_, next)| is_word_character(*next));

            if is_word_character(character) || connector_between_word_characters {
                end = byte + character.len_utf8();
                continue;
            }

            self.cursor = byte + character.len_utf8();
            return Some(TextSpan {
                text: &self.chunk.text[relative_start..end],
                start_byte: self.chunk.start_byte + relative_start,
                end_byte: self.chunk.start_byte + end,
            });
        }

        self.cursor = self.chunk.text.len();
        if end > relative_start {
            Some(TextSpan {
                text: &self.chunk.text[relative_start..end],
                start_byte: self.chunk.start_byte + relative_start,
                end_byte: self.chunk.start_byte + end,
            })
        } else {
            None
        }
    }
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric()
        || matches!(
            get_general_category(character),
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
        )
}

fn is_connector(character: char) -> bool {
    matches!(character, '\'' | '\u{2019}' | '-' | '\u{2010}' | '\u{2011}')
}

fn looks_like_email_or_url(chunk: &str) -> bool {
    let candidate = chunk.trim_matches(|character: char| {
        !character.is_alphanumeric() && !matches!(character, '.' | '/' | ':' | '@' | '-' | '_')
    });

    if candidate.contains('@')
        || starts_with_ascii_case_insensitive(candidate, "http://")
        || starts_with_ascii_case_insensitive(candidate, "https://")
        || starts_with_ascii_case_insensitive(candidate, "www.")
        || starts_with_ascii_case_insensitive(candidate, "mailto:")
    {
        return true;
    }

    if candidate.contains('/') && (candidate.contains('.') || candidate.contains(':')) {
        return true;
    }

    let Some((host, suffix)) = candidate.rsplit_once('.') else {
        return false;
    };

    !host.is_empty()
        && host.chars().any(char::is_alphabetic)
        && (2..=24).contains(&suffix.len())
        && suffix
            .chars()
            .all(|character| character.is_ascii_alphabetic())
}

fn starts_with_ascii_case_insensitive(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn normalize(word: &str) -> String {
    normalize_for_check(word).into_owned()
}

fn normalize_for_check(word: &str) -> Cow<'_, str> {
    if word.chars().any(char::is_uppercase) {
        Cow::Owned(word.to_lowercase())
    } else {
        Cow::Borrowed(word)
    }
}

pub(crate) fn normalize_allow_word(word: &str) -> Option<String> {
    if word.is_empty()
        || word.len() > MAX_WORD_LEN
        || word.chars().any(char::is_numeric)
        || !word.chars().any(char::is_alphabetic)
    {
        return None;
    }

    let whole = TextSpan {
        text: word,
        start_byte: 0,
        end_byte: word.len(),
    };
    let mut parsed = tokens(whole);
    let token = parsed.next()?;
    if token.start_byte != 0 || token.end_byte != word.len() || parsed.next().is_some() {
        return None;
    }
    Some(normalize(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_dictionaries_parse_and_accept_both_languages() {
        let dictionaries = BilingualDictionaries::new().expect("dictionaries parse");

        assert!(dictionaries.en_us.check("experience"));
        assert!(dictionaries.sv_se.check("erfarenhet"));
    }

    #[test]
    fn reports_unique_unknown_terms_in_source_order_with_byte_spans() {
        let blocks = ["bra blåqzx Franglemist", "blåqzx quuxzorp"];
        let issues = check_blocks(&blocks, &[], usize::MAX);

        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].term, "blåqzx");
        assert_eq!(issues[0].normalized_term, "blåqzx");
        assert_eq!(issues[0].block_index, 0);
        assert_eq!(issues[0].block_start_byte, 4);
        assert_eq!(issues[0].block_end_byte, 11);
        assert_eq!(
            issues[0].classification,
            SpellingClassification::UnknownWord
        );

        assert_eq!(issues[1].term, "Franglemist");
        assert_eq!(
            issues[1].classification,
            SpellingClassification::PossibleName
        );
        assert_eq!(issues[2].term, "quuxzorp");
        assert_eq!(issues[2].block_index, 1);
    }

    #[test]
    fn skips_email_url_digit_and_oversized_tokens() {
        let oversized = "q".repeat(MAX_WORD_LEN + 1);
        let text = format!(
            "wrong@example.com https://qzx.invalid www.qzx.invalid ISO27001 {oversized} quuxzorp"
        );
        let issues = check_blocks(&[text.as_str()], &[], usize::MAX);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].term, "quuxzorp");
    }

    #[test]
    fn developer_and_user_allowlists_are_case_insensitive() {
        let user_allow_words = vec!["QuuxZorp".to_owned()];
        let issues = check_blocks(
            &["Kubernetes QUUXZORP quuxzorp"],
            &user_allow_words,
            usize::MAX,
        );

        assert!(issues.is_empty());
    }

    #[test]
    fn allow_word_normalization_requires_one_checkable_token() {
        assert_eq!(
            normalize_allow_word("Quux-Zorp").as_deref(),
            Some("quux-zorp")
        );
        assert!(normalize_allow_word("quuxzorp,").is_none());
        assert!(normalize_allow_word("two words").is_none());
        assert!(normalize_allow_word("ISO27001").is_none());
        assert!(normalize_allow_word("---").is_none());
    }

    #[test]
    fn tokenizes_internal_apostrophes_and_hyphens_without_wrapping_punctuation() {
        let mut chunks = chunks("(engineer's) qzx-word.");
        let first = chunks.next().expect("first chunk");
        let second = chunks.next().expect("second chunk");
        let spans: Vec<_> = tokens(first).chain(tokens(second)).collect();

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "engineer's");
        assert_eq!((spans[0].start_byte, spans[0].end_byte), (1, 11));
        assert_eq!(spans[1].text, "qzx-word");
        assert_eq!((spans[1].start_byte, spans[1].end_byte), (13, 21));
    }

    #[test]
    fn issue_collection_stops_at_the_requested_limit() {
        let issues = check_blocks(&["quuxzorp blorfzle snazzwump"], &[], 2);

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].term, "quuxzorp");
        assert_eq!(issues[1].term, "blorfzle");
    }
}
