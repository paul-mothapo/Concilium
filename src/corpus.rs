use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CorpusLoadReport {
    pub files: Vec<PathBuf>,
    pub glosses: Vec<String>,
    pub sentences: Vec<String>,
    pub api_sources: Vec<String>,
}

impl CorpusLoadReport {
    pub fn merge(mut self, other: Self) -> Self {
        self.files.extend(other.files);
        self.api_sources.extend(other.api_sources);

        let mut glosses = BTreeSet::new();
        glosses.extend(self.glosses);
        glosses.extend(other.glosses);

        let mut sentences = BTreeSet::new();
        sentences.extend(self.sentences);
        sentences.extend(other.sentences);

        self.glosses = glosses.into_iter().collect();
        self.sentences = sentences.into_iter().collect();
        self.files.sort();
        self.files.dedup();
        self.api_sources.sort();
        self.api_sources.dedup();
        self
    }

    pub fn limit(mut self, max_glosses: usize, max_sentences: usize) -> Self {
        if self.glosses.len() > max_glosses {
            self.glosses.truncate(max_glosses);
        }

        if self.sentences.len() > max_sentences {
            self.sentences.truncate(max_sentences);
        }

        self
    }
}

pub fn load_glosses_from_data_dir(path: &Path) -> Result<Vec<String>, String> {
    Ok(load_corpus_from_data_dir(path)?.glosses)
}

pub fn load_paragraphs_from_markdown(path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(parse_markdown_paragraphs(&content))
}

pub fn load_corpus_from_data_dir(path: &Path) -> Result<CorpusLoadReport, String> {
    let mut words = BTreeSet::new();
    let mut sentences = BTreeSet::new();
    let mut files = Vec::new();

    collect_supported_files(path, &mut files)?;
    files.sort();

    for file_path in &files {
        let Some(extension) = file_path
            .extension()
            .and_then(|extension| extension.to_str())
        else {
            continue;
        };

        let content = fs::read_to_string(file_path)
            .map_err(|error| format!("failed to read {}: {error}", file_path.display()))?;

        let extracted = match extension {
            "md" => parse_markdown_content(&content),
            "json" => parse_json_content(&content)?,
            _ => ParsedCorpusContent::default(),
        };

        words.extend(extracted.words);
        sentences.extend(extracted.sentences);
    }

    if files.is_empty() {
        return Err(format!(
            "no supported .md or .json corpus files were found in {}",
            path.display()
        ));
    }

    Ok(CorpusLoadReport {
        files,
        glosses: words.into_iter().collect(),
        sentences: sentences.into_iter().collect(),
        api_sources: Vec::new(),
    })
}

fn collect_supported_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("failed to read data directory {}: {error}", path.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read a directory entry: {error}"))?;
        let file_path = entry.path();

        if file_path.is_dir() {
            collect_supported_files(&file_path, files)?;
            continue;
        }

        let Some(extension) = file_path
            .extension()
            .and_then(|extension| extension.to_str())
        else {
            continue;
        };

        if matches!(extension, "md" | "json") {
            files.push(file_path);
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ParsedCorpusContent {
    words: Vec<String>,
    sentences: Vec<String>,
}

fn parse_markdown_content(content: &str) -> ParsedCorpusContent {
    let mut words = Vec::new();
    let mut sentences = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if looks_like_sentence(trimmed) {
            sentences.push(trimmed.to_owned());
        }
        words.extend(extract_words(trimmed));
    }

    ParsedCorpusContent { words, sentences }
}

fn parse_markdown_paragraphs(content: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }

        if trimmed.starts_with('#') {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }

        current.push(trimmed.to_owned());
    }

    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }

    paragraphs
}

fn parse_json_content(content: &str) -> Result<ParsedCorpusContent, String> {
    let strings = extract_json_string_values(content)?;
    let mut words = Vec::new();
    let mut sentences = Vec::new();

    for value in strings {
        if looks_like_sentence(&value) {
            sentences.push(value.clone());
        }
        words.extend(extract_words(&value));
    }

    Ok(ParsedCorpusContent { words, sentences })
}

fn extract_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for character in text.chars() {
        if character.is_ascii_alphabetic() || character == '\'' {
            current.push(character.to_ascii_lowercase());
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn extract_json_string_values(content: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut content_chars = content.chars().peekable();

    while let Some(character) = content_chars.next() {
        if !in_string {
            if character == '"' {
                in_string = true;
                current.clear();
            }
            continue;
        }

        if escaped {
            current.push(match character {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => {
                in_string = false;

                let mut next_significant = None;
                while let Some(peeked) = content_chars.peek().copied() {
                    if peeked.is_whitespace() {
                        content_chars.next();
                        continue;
                    }

                    next_significant = Some(peeked);
                    break;
                }

                let is_key = next_significant == Some(':');
                if !is_key {
                    values.push(current.clone());
                }
            }
            other => current.push(other),
        }
    }

    if in_string {
        return Err("invalid json: unterminated string".to_owned());
    }

    Ok(values)
}

fn looks_like_sentence(text: &str) -> bool {
    let has_sentence_punctuation = text.ends_with('.')
        || text.ends_with('?')
        || text.ends_with('!')
        || text.ends_with(':')
        || text.ends_with(';');

    has_sentence_punctuation || text.split_whitespace().count() > 1
}

#[cfg(test)]
mod tests {
    use super::{extract_json_string_values, parse_markdown_content};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parses_markdown_word_lists_and_sentences() {
        let content = "# Heading\n\nI see you.\nThis is good.\n";
        let parsed = parse_markdown_content(content);

        assert!(parsed.words.iter().any(|word| word == "i"));
        assert!(parsed.words.iter().any(|word| word == "see"));
        assert!(parsed.words.iter().any(|word| word == "good"));
        assert!(
            parsed
                .sentences
                .iter()
                .any(|sentence| sentence == "I see you.")
        );
    }

    #[test]
    fn parses_json_values_without_including_keys() {
        let content = r#"{"words":["I","you"],"sentences":["I see you."]}"#;
        let values = extract_json_string_values(content).expect("json should parse");

        assert_eq!(values, vec!["I", "you", "I see you."]);
    }

    #[test]
    fn loads_markdown_and_json_recursively() {
        let root = PathBuf::from("target/test-corpus-recursive");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("test directory should be created");
        fs::write(root.join("words.md"), "# words\nalpha\n").expect("markdown should be written");
        fs::write(nested.join("more.json"), r#"{"items":["beta gamma"]}"#)
            .expect("json should be written");

        let report = super::load_corpus_from_data_dir(&root).expect("corpus should load");

        assert_eq!(report.files.len(), 2);
        assert!(report.glosses.iter().any(|word| word == "alpha"));
        assert!(report.glosses.iter().any(|word| word == "beta"));
        assert!(report.glosses.iter().any(|word| word == "gamma"));
        assert!(
            report
                .sentences
                .iter()
                .any(|sentence| sentence == "beta gamma")
        );

        fs::remove_dir_all(&root).expect("test directory should be removed");
    }
}
