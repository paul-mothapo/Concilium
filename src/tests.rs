use crate::corpus::load_corpus_from_data_dir;
use crate::evolution::{GlossSyntaxNode, LanguageEngine};
use crate::semantics::{Concept};
use crate::form::WordForm;
use crate::glossary::render_english_to_concilium;
use crate::grammar::{Grammar, PhraseCategory, WordOrder, FeatureValue, MorphologyEngine, ParadigmRule, SyntaxNode};
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
    let morphology = MorphologyEngine {
        rules: vec![
            ParadigmRule {
                feature: FeatureValue::Plural,
                prefix: None,
                suffix: Some(vec!["e".to_owned(), "n".to_owned()]),
            },
            ParadigmRule {
                feature: FeatureValue::Past,
                prefix: Some(vec!["k".to_owned(), "a".to_owned()]),
                suffix: None,
            },
        ],
    };
    let grammar = Grammar::new(WordOrder::SOV, morphology);
    
    let node = SyntaxNode::branch(
        PhraseCategory::Sentence,
        vec![
            SyntaxNode::leaf(WordForm::new(["mi"])),
            SyntaxNode::leaf(WordForm::new(["tar"])),
            SyntaxNode::leaf(WordForm::new(["ven"])),
        ],
    );

    let rendered = grammar.render_node(&node, &[FeatureValue::Plural, FeatureValue::Past]);
    // SOV -> mi taren kaven (assuming plural applies to object[1]? No, currently apply() applies to all leaves)
    // Wait, in my current implementation of realize_node, features are passed down to ALL leaves.
    // This is fine for now as a "greedy" morphology.
    assert_eq!(rendered, "mi taren kaven");
}

#[test]
fn language_generation_preserves_lexicon_size_and_demo_sentence() {
    let engine = LanguageEngine;
    let blueprint = concilium_blueprint();
    let mut rng = Random::new(29);

    let language = engine.generate_language(
        &blueprint,
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
fn recursive_syntax_renders_nested_structures() {
    let engine = LanguageEngine;
    let mut blueprint = concilium_blueprint();
    for gloss in &["i", "you", "see", "tree", "big"] {
        if blueprint.semantic_mapper.resolve_gloss(gloss).is_empty() {
            let concept = Concept::new(*gloss, *gloss);
            let id = concept.id.clone();
            blueprint.semantic_mapper.add_concept(concept);
            blueprint.semantic_mapper.map_gloss(*gloss, id);
        }
    }

    let mut rng = Random::new(42);
    let language = engine.generate_language(
        &blueprint,
        WordGenerationConfig::new(1, 2),
        &mut rng,
    );

    // [Sentence: [Big Tree] [I] [See]] -> "Big Tree I See" (assuming SOV and flat NP)
    let tree = GlossSyntaxNode::branch(
        PhraseCategory::Sentence,
        vec![
            GlossSyntaxNode::branch(
                PhraseCategory::NounPhrase,
                vec![
                    GlossSyntaxNode::leaf("big"),
                    GlossSyntaxNode::leaf("tree"),
                ],
            ),
            GlossSyntaxNode::leaf("i"),
            GlossSyntaxNode::leaf("see"),
        ],
    );

    let rendered = language.render_tree_from_glosses(&tree).unwrap();
    
    // In SOV, order is S O V. 
    // Here we passed NP, I, See. 
    // NP should be O? No, order_sentence in grammar.rs uses indices: children[0]=S, children[1]=O, children[2]=V.
    // So if children[0] is NP, it is the Subject.
    
    let big = language.lexemes_for_gloss("big").into_iter().next().unwrap().form.text();
    let tree_word = language.lexemes_for_gloss("tree").into_iter().next().unwrap().form.text();
    let i = language.lexemes_for_gloss("i").into_iter().next().unwrap().form.text();
    let see = language.lexemes_for_gloss("see").into_iter().next().unwrap().form.text();

    // Expected: "Big Tree I See" (if Big Tree is Subject)
    assert_eq!(rendered, format!("{} {} {} {}", big, tree_word, i, see));
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
