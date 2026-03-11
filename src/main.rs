use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use concilium_language_engine::LanguageEngine;
use concilium_language_engine::corpus::load_corpus_from_data_dir;
use concilium_language_engine::glossary::render_english_to_concilium;
use concilium_language_engine::presets::{concilium_blueprint, demo_generation_config, demo_rng};
use concilium_language_engine::public_api::fetch_public_api_corpus;

fn main() {
    const CONTEXT_WINDOW: usize = 10000;

    let mut rng = demo_rng();
    let engine = LanguageEngine;
    let blueprint = concilium_blueprint();
    let input_dir = Path::new("data");
    let local_output_path = Path::new("Transalation.md");
    let longer_output_path = next_longer_output_path();
    let local_corpus = load_corpus_from_data_dir(input_dir).expect("failed to load corpus data");
    let local_gloss_refs = local_corpus
        .glosses
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let local_language = engine.generate_language(
        &blueprint,
        &local_gloss_refs,
        demo_generation_config(),
        &mut rng,
    );
    let local_glossary =
        dedupe_glossary_markdown(&render_english_to_concilium(&local_language, &local_corpus));
    fs::write(local_output_path, local_glossary).expect("failed to write Transalation.md");

    let public_api_corpus = fetch_public_api_corpus(CONTEXT_WINDOW);
    let longer_corpus = match public_api_corpus {
        Ok(remote) => local_corpus
            .clone()
            .merge(remote)
            .limit(CONTEXT_WINDOW, CONTEXT_WINDOW),
        Err(error) => {
            eprintln!("Public API fetch skipped: {error}");
            local_corpus.clone().limit(CONTEXT_WINDOW, CONTEXT_WINDOW)
        }
    };
    let all_gloss_refs = longer_corpus
        .glosses
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let existing_glosses = load_existing_english_glosses();
    let filtered_gloss_refs = all_gloss_refs
        .iter()
        .copied()
        .filter(|gloss| !existing_glosses.contains(*gloss))
        .collect::<Vec<_>>();
    let gloss_refs = if filtered_gloss_refs.is_empty() {
        all_gloss_refs
    } else {
        filtered_gloss_refs
    };
    let language =
        engine.generate_language(&blueprint, &gloss_refs, demo_generation_config(), &mut rng);
    let raw_glossary = render_english_to_concilium(&language, &longer_corpus);
    let glossary = filter_new_glossary_markdown(&raw_glossary, &existing_glosses);

    fs::write(&longer_output_path, glossary)
        .expect("failed to write longer translation output");

    print_language(&language);
    println!("Input Directory: {}", input_dir.display());
    println!("Context Window Limit: {}", CONTEXT_WINDOW);
    println!("Loaded Files: {}", longer_corpus.files.len());
    for file in &longer_corpus.files {
        println!("Loaded Corpus File: {}", file.display());
    }
    println!("Loaded Public APIs: {}", longer_corpus.api_sources.len());
    for source in &longer_corpus.api_sources {
        println!("Loaded Public API: {}", source);
    }
    println!("Output File: {}", local_output_path.display());
    println!("API Output File: {}", longer_output_path.display());
    println!("Translated Words: {}", language.lexicon.len());
}

fn dedupe_glossary_markdown(glossary: &str) -> String {
    let mut seen = HashSet::new();
    let mut result = String::new();
    let mut in_header = true;

    for line in glossary.lines() {
        let trimmed = line.trim_start();

        // Always keep non-table lines and the header (first 4 lines of the standard table)
        if in_header {
            result.push_str(line);
            result.push('\n');
            if trimmed.starts_with('|') && trimmed.contains("Pronunciation") {
                // Next separator line will be the last header line
                in_header = false;
            }
            continue;
        }

        // Keep the separator row as-is
        if trimmed.starts_with('|') && trimmed.contains("---") {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Only process table rows that look like markdown table entries
        if !trimmed.starts_with('|') {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let parts: Vec<_> = line.split('|').map(str::trim).collect();
        if parts.len() < 4 {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let english = parts[1];
        if seen.insert(english.to_owned()) {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

fn filter_new_glossary_markdown(glossary: &str, existing: &HashSet<String>) -> String {
    let mut seen_in_file = HashSet::new();
    let mut result = String::new();
    let mut in_header = true;

    for line in glossary.lines() {
        let trimmed = line.trim_start();

        // Always keep non-table lines and the header (first 4 lines of the standard table)
        if in_header {
            result.push_str(line);
            result.push('\n');
            if trimmed.starts_with('|') && trimmed.contains("Pronunciation") {
                // Next separator line will be the last header line
                in_header = false;
            }
            continue;
        }

        // Keep the separator row as-is
        if trimmed.starts_with('|') && trimmed.contains("---") {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Only process table rows that look like markdown table entries
        if !trimmed.starts_with('|') {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let parts: Vec<_> = line.split('|').map(str::trim).collect();
        if parts.len() < 4 {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let english = parts[1];
        if english.is_empty()
            || existing.contains(english)
            || !seen_in_file.insert(english.to_owned())
        {
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

fn load_existing_english_glosses() -> HashSet<String> {
    let mut seen = HashSet::new();

    let is_translation_file = |name: &str| {
        name == "Transalation.md"
            || name == "transaltion_longer.md"
            || (name.starts_with("translation_longer_v") && name.ends_with(".md"))
    };

    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if !is_translation_file(name) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for line in content.lines() {
                let trimmed = line.trim_start();
                if !trimmed.starts_with('|') {
                    continue;
                }
                if trimmed.starts_with("| English |") || trimmed.starts_with("| ---") {
                    continue;
                }

                let parts: Vec<_> = line.split('|').map(str::trim).collect();
                if parts.len() < 4 {
                    continue;
                }

                let english = parts[1];
                if !english.is_empty() {
                    seen.insert(english.to_owned());
                }
            }
        }
    }

    seen
}

fn next_longer_output_path() -> PathBuf {
    let base = Path::new("transaltion_longer.md");
    if !base.exists() {
        return base.to_path_buf();
    }

    let mut max_index = 1usize;
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if !file_type.is_file() {
                continue;
            }

            let name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };

            if let Some(rest) = name.strip_prefix("translation_longer_v") {
                if let Some(number_str) = rest.strip_suffix(".md") {
                    if let Ok(n) = number_str.parse::<usize>() {
                        if n > max_index {
                            max_index = n;
                        }
                    }
                }
            }
        }
    }

    let next_index = max_index + 1;
    PathBuf::from(format!("translation_longer_v{}.md", next_index))
}

fn print_language(language: &concilium_language_engine::Language) {
    let phonemes = language.inventory_snapshot().join(", ");
    let words = language
        .sample_words(4)
        .into_iter()
        .map(|word| word.text())
        .collect::<Vec<_>>()
        .join(", ");
    let translation = language
        .render_clause_from_glosses("i", "you", "see", false, false)
        .unwrap_or_else(|| "missing lexemes".to_owned());

    println!("Language Name: {}", language.name);
    println!("Phonemes: {}", phonemes);
    println!("Example words: {}", words);
    println!("Translation (I see you): {}", translation);
}
