use crate::corpus::load_corpus_from_data_dir;
use crate::evolution::LanguageEngine;
use crate::form::WordForm;
use crate::glossary::render_english_to_concilium;
use crate::grammar::{Clause, Grammar, WordOrder};
use crate::lexicon::{LexiconGenerator, WordGenerationConfig};
use crate::mutation::{Environment, Matcher, SoundChange};
use crate::phonology::{
    PhonemeClass, Phonology, PhonotacticConstraints, Slot, SyllableTemplate, WeightedPhoneme,
};
use crate::presets::{DEMO_GLOSSES, concilium_blueprint, demo_generation_config};
use crate::rng::Random;
use std::path::Path;

#[test]
fn generated_words_respect_initial_consonant_constraint() {
    let blueprint = concilium_blueprint();
    let generator = LexiconGenerator::new(&blueprint.phonology);
    let mut rng = Random::new(17);

    for _ in 0..64 {
        let word = generator.generate_word(demo_generation_config(), &mut rng);
        assert!(blueprint.phonology.allows(&word), "invalid word: {word}");
    }
}

#[test]
fn sound_change_applies_only_in_matching_environment() {
    let phonology = simple_phonology();
    let change = SoundChange::new(
        "brighten a",
        "a",
        vec!["ae"],
        1.0,
        Environment::between(Matcher::Consonant, Matcher::Consonant),
    );

    let changed = change.apply(
        &WordForm::new(["t", "a", "k"]),
        &phonology,
        &mut Random::new(9),
    );
    let unchanged = change.apply(&WordForm::new(["a", "k"]), &phonology, &mut Random::new(9));

    assert_eq!(changed.text(), "taek");
    assert_eq!(unchanged.text(), "ak");
}

#[test]
fn grammar_realizes_word_order_and_inflection() {
    let grammar = Grammar::new(WordOrder::SOV, Some(vec!["e", "n"]), Some(vec!["k", "a"]));
    let clause = Clause::new(
        WordForm::new(["mi"]),
        WordForm::new(["tar"]),
        WordForm::new(["ven"]),
    )
    .with_plural_object()
    .with_past_verb();

    assert_eq!(grammar.render_clause(&clause), "mi taren kaven");
}

#[test]
fn language_generation_preserves_lexicon_size_and_demo_sentence() {
    let engine = LanguageEngine;
    let blueprint = concilium_blueprint();
    let mut rng = Random::new(29);

    let language = engine.generate_language(
        &blueprint,
        DEMO_GLOSSES,
        WordGenerationConfig::new(1, 2),
        &mut rng,
    );

    assert_eq!(language.name, "Concilium");
    assert_eq!(language.lexicon.len(), DEMO_GLOSSES.len());
    assert!(
        language
            .render_clause_from_glosses("i", "tree", "see", true, true)
            .is_some()
    );
    assert!(
        language
            .render_clause_from_glosses("i", "you", "see", false, false)
            .is_some()
    );
    assert!(
        language
            .inventory_snapshot()
            .iter()
            .any(|symbol| symbol == "kh")
    );

    let corpus = load_corpus_from_data_dir(Path::new("data")).expect("data directory should load");
    let glossary = render_english_to_concilium(&language, &corpus);
    assert!(glossary.contains("# English to Concilium: Words"));
    assert!(glossary.contains("["));
    assert!(glossary.contains("# English to Concilium: Sentences"));
}

#[test]
fn corpus_loader_reads_markdown_data_directory() {
    let corpus = load_corpus_from_data_dir(Path::new("data")).expect("data directory should load");

    assert!(!corpus.files.is_empty());
    assert!(corpus.glosses.iter().any(|word| word == "i"));
    assert!(corpus.glosses.iter().any(|word| word == "you"));
    assert!(corpus.glosses.iter().any(|word| word == "see"));
    assert!(!corpus.sentences.is_empty());
    assert!(corpus.api_sources.is_empty());
}

fn simple_phonology() -> Phonology {
    Phonology::new(
        vec![
            WeightedPhoneme::new("a", 1),
            WeightedPhoneme::new("e", 1),
            WeightedPhoneme::new("i", 1),
        ],
        vec![
            WeightedPhoneme::new("t", 1),
            WeightedPhoneme::new("k", 1),
            WeightedPhoneme::new("m", 1),
        ],
        vec![WeightedPhoneme::new("tr", 1)],
        vec![WeightedPhoneme::new("k", 1), WeightedPhoneme::new("n", 1)],
        vec![SyllableTemplate::new(
            "CV",
            vec![
                Slot::required(PhonemeClass::OnsetConsonant),
                Slot::required(PhonemeClass::Vowel),
            ],
            1,
        )],
        PhonotacticConstraints::new(false),
    )
}
