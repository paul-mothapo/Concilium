use concilium_language_engine::LanguageEngine;
use concilium_language_engine::presets::{
    DEMO_GLOSSES, concilium_blueprint, demo_generation_config, demo_rng,
};

fn main() {
    let mut rng = demo_rng();
    let engine = LanguageEngine;
    let blueprint = concilium_blueprint();
    let language =
        engine.generate_language(&blueprint, DEMO_GLOSSES, demo_generation_config(), &mut rng);

    print_language(&language);
}

fn print_language(language: &concilium_language_engine::Language) {
    let phonemes = language.inventory_snapshot().join(", ");
    let words = language
        .sample_words(4)
        .into_iter()
        .map(|word| word.text())
        .collect::<Vec<_>>()
        .join(", ");
    let sentence = language
        .render_clause_from_glosses("i", "tree", "see", true, true)
        .unwrap_or_else(|| "missing lexemes".to_owned());
    let translation = language
        .render_clause_from_glosses("i", "you", "see", false, false)
        .unwrap_or_else(|| "missing lexemes".to_owned());

    println!("Language Name: {}", language.name);
    println!("Phonemes: {}", phonemes);
    println!("Example words: {}", words);
    println!("Sentence: {}", sentence);
    println!("Translation (I see you): {}", translation);
}
