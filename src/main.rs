use std::fs;
use std::path::Path;

use concilium_language_engine::LanguageEngine;
use concilium_language_engine::corpus::load_corpus_from_data_dir;
use concilium_language_engine::glossary::render_english_to_concilium;
use concilium_language_engine::presets::{concilium_blueprint, demo_generation_config, demo_rng};

fn main() {
    let mut rng = demo_rng();
    let engine = LanguageEngine;
    let blueprint = concilium_blueprint();
    let input_dir = Path::new("data");
    let output_path = Path::new("Transalation.md");
    let corpus = load_corpus_from_data_dir(input_dir).expect("failed to load corpus data");
    let gloss_refs = corpus
        .glosses
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let language =
        engine.generate_language(&blueprint, &gloss_refs, demo_generation_config(), &mut rng);
    let glossary = render_english_to_concilium(&language, &corpus);

    fs::write(output_path, glossary).expect("failed to write Transalation.md");

    print_language(&language);
    println!("Input Directory: {}", input_dir.display());
    println!("Loaded Files: {}", corpus.files.len());
    for file in &corpus.files {
        println!("Loaded Corpus File: {}", file.display());
    }
    println!("Output File: {}", output_path.display());
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
