use std::env;
use std::fs;
use std::path::Path;
use std::process;

use concilium_language_engine::LanguageEngine;
use concilium_language_engine::corpus::load_corpus_from_data_dir;
use concilium_language_engine::glossary::{render_lexicon_markdown, render_sentences_markdown};
use concilium_language_engine::presets::{concilium_blueprint, demo_generation_config, demo_rng};

enum RunMode {
    Words,
    Sentences,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mode = if args.len() < 2 {
        print_usage();
        process::exit(0);
    } else {
        match args[1].as_str() {
            "words" => RunMode::Words,
            "sentences" => RunMode::Sentences,
            _ => {
                eprintln!("Error: Unknown mode '{}'", args[1]);
                print_usage();
                process::exit(1);
            }
        }
    };

    let mut rng = demo_rng();
    let engine = LanguageEngine;
    let mut blueprint = concilium_blueprint();
    let input_dir = Path::new("data");

    let local_corpus = load_corpus_from_data_dir(input_dir).expect("failed to load corpus data");
    
    // Ensure all glosses in the corpus are mapped to concepts in the blueprint
    for gloss in &local_corpus.glosses {
        if blueprint.semantic_mapper.resolve_gloss(gloss).is_empty() {
            let concept = concilium_language_engine::Concept::new(gloss, gloss);
            let id = concept.id.clone();
            blueprint.semantic_mapper.add_concept(concept);
            blueprint.semantic_mapper.map_gloss(gloss, id);
        }
    }

    let language = engine.generate_language(
        &blueprint,
        demo_generation_config(),
        &mut rng,
    );

    match mode {
        RunMode::Words => {
            let output_path = Path::new("Words.md");
            let content = render_lexicon_markdown(&language);
            fs::write(output_path, content).expect("failed to write Words.md");
            println!("Lexicon generated in: {}", output_path.display());
        }
        RunMode::Sentences => {
            let output_path = Path::new("Sentences.md");
            let content = render_sentences_markdown(&language, &local_corpus);
            fs::write(output_path, content).expect("failed to write Sentences.md");
            println!("Sentences translated in: {}", output_path.display());
        }
    }

    print_language_summary(&language);
}

fn print_usage() {
    println!("Concilium Language Engine");
    println!("Usage: cargo run -- <mode>");
    println!("");
    println!("Modes:");
    println!("  words      Generate the word lexicon in Words.md");
    println!("  sentences  Translate corpus sentences in Sentences.md");
}

fn print_language_summary(language: &concilium_language_engine::Language) {
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

    println!("---");
    println!("Language Name: {}", language.name);
    println!("Phonemes: {}", phonemes);
    println!("Example words: {}", words);
    println!("Translation (I see you): {}", translation);
}
