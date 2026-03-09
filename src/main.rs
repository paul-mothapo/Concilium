use std::fs;
use std::path::Path;

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
    let longer_output_path = Path::new("transaltion_longer.md");
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
    let local_glossary = render_english_to_concilium(&local_language, &local_corpus);
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
    let gloss_refs = longer_corpus
        .glosses
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let language =
        engine.generate_language(&blueprint, &gloss_refs, demo_generation_config(), &mut rng);
    let glossary = render_english_to_concilium(&language, &longer_corpus);

    fs::write(longer_output_path, glossary).expect("failed to write transaltion_longer.md");

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
