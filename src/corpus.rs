use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn load_glosses_from_data_dir(path: &Path) -> Result<Vec<String>, String> {
    let mut words = BTreeSet::new();

    let entries = fs::read_dir(path)
        .map_err(|error| format!("failed to read data directory {}: {error}", path.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read a directory entry: {error}"))?;
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let Some(extension) = file_path
            .extension()
            .and_then(|extension| extension.to_str())
        else {
            continue;
        };

        let content = fs::read_to_string(&file_path)
            .map_err(|error| format!("failed to read {}: {error}", file_path.display()))?;

        let extracted = match extension {
            "md" => parse_markdown_words(&content),
            "json" => parse_json_words(&content)?,
            _ => Vec::new(),
        };

        words.extend(extracted);
    }

    if words.is_empty() {
        return Err(format!(
            "no supported .md or .json corpus files were found in {}",
            path.display()
        ));
    }

    Ok(words.into_iter().collect())
}

fn parse_markdown_words(content: &str) -> Vec<String> {
    let mut words = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        words.extend(extract_words(trimmed));
    }

    words
}

fn parse_json_words(content: &str) -> Result<Vec<String>, String> {
    let strings = extract_json_string_values(content)?;
    let mut words = Vec::new();

    for value in strings {
        words.extend(extract_words(&value));
    }

    Ok(words)
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

#[cfg(test)]
mod tests {
    use super::{extract_json_string_values, parse_markdown_words};

    #[test]
    fn parses_markdown_word_lists_and_sentences() {
        let content = "# Heading\n\nI see you.\nThis is good.\n";
        let words = parse_markdown_words(content);

        assert!(words.iter().any(|word| word == "i"));
        assert!(words.iter().any(|word| word == "see"));
        assert!(words.iter().any(|word| word == "good"));
    }

    #[test]
    fn parses_json_values_without_including_keys() {
        let content = r#"{"words":["I","you"],"sentences":["I see you."]}"#;
        let values = extract_json_string_values(content).expect("json should parse");

        assert_eq!(values, vec!["I", "you", "I see you."]);
    }
}
