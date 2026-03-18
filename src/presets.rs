use crate::evolution::LanguageBlueprint;
use crate::grammar::{Grammar, WordOrder, FeatureValue, MorphologyEngine, ParadigmRule};
use crate::semantics::{Concept, SemanticMapper};
use crate::lexicon::WordGenerationConfig;
use crate::mutation::{Environment, Matcher, SoundChange};
use crate::phonology::{
    PhonemeClass, Phonology, PhonotacticConstraints, Slot, SyllableTemplate, WeightedPhoneme, PhonemeFeature,
};
use crate::rng::Random;

pub const DEMO_GLOSSES: &[&str] = &[
    "i", "you", "see", "tree", "river", "stone", "sky", "fire", "song", "king", "moon",
];

pub fn demo_seed() -> u64 {
    0x00C0_1C11_A55A_u64
}

pub fn demo_generation_config() -> WordGenerationConfig {
    WordGenerationConfig::new(1, 2)
}

pub fn demo_rng() -> Random {
    Random::new(demo_seed())
}

pub fn concilium_blueprint() -> LanguageBlueprint {
    let vowels = vec![
        WeightedPhoneme::new("a", 5).with_features(vec![PhonemeFeature::Low, PhonemeFeature::Back]).with_ipa("ɑ"),
        WeightedPhoneme::new("e", 4).with_features(vec![PhonemeFeature::Front]).with_ipa("e"),
        WeightedPhoneme::new("i", 3).with_features(vec![PhonemeFeature::High, PhonemeFeature::Front]).with_ipa("i"),
        WeightedPhoneme::new("o", 3).with_features(vec![PhonemeFeature::Back]).with_ipa("o"),
        WeightedPhoneme::new("u", 2).with_features(vec![PhonemeFeature::High, PhonemeFeature::Back]).with_ipa("u"),
    ];

    let onset_consonants = vec![
        WeightedPhoneme::new("l", 4).with_features(vec![PhonemeFeature::Approximant, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("l"),
        WeightedPhoneme::new("m", 4).with_features(vec![PhonemeFeature::Nasal, PhonemeFeature::Labial, PhonemeFeature::Voiced]).with_ipa("m"),
        WeightedPhoneme::new("n", 4).with_features(vec![PhonemeFeature::Nasal, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("n"),
        WeightedPhoneme::new("r", 4).with_features(vec![PhonemeFeature::Approximant, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("r"),
        WeightedPhoneme::new("k", 3).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Velar, PhonemeFeature::Voiceless]).with_ipa("k"),
        WeightedPhoneme::new("t", 3).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Alveolar, PhonemeFeature::Voiceless]).with_ipa("t"),
        WeightedPhoneme::new("g", 2).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Velar, PhonemeFeature::Voiced]).with_ipa("ɡ"),
        WeightedPhoneme::new("d", 2).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("d"),
        WeightedPhoneme::new("s", 3).with_features(vec![PhonemeFeature::Fricative, PhonemeFeature::Alveolar, PhonemeFeature::Voiceless]).with_ipa("s"),
        WeightedPhoneme::new("v", 2).with_features(vec![PhonemeFeature::Fricative, PhonemeFeature::Labial, PhonemeFeature::Voiced]).with_ipa("v"),
    ];

    let clusters = vec![
        WeightedPhoneme::new("dr", 2).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("dɹ"),
        WeightedPhoneme::new("kr", 2).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Velar, PhonemeFeature::Voiceless]).with_ipa("kɹ"),
        WeightedPhoneme::new("sh", 2).with_features(vec![PhonemeFeature::Fricative, PhonemeFeature::Palatal, PhonemeFeature::Voiceless]).with_ipa("ʃ"),
        WeightedPhoneme::new("tl", 1).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Alveolar, PhonemeFeature::Voiceless]).with_ipa("tl"),
    ];

    let coda_consonants = vec![
        WeightedPhoneme::new("l", 2).with_features(vec![PhonemeFeature::Approximant, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("l"),
        WeightedPhoneme::new("n", 3).with_features(vec![PhonemeFeature::Nasal, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("n"),
        WeightedPhoneme::new("r", 3).with_features(vec![PhonemeFeature::Approximant, PhonemeFeature::Alveolar, PhonemeFeature::Voiced]).with_ipa("r"),
        WeightedPhoneme::new("k", 2).with_features(vec![PhonemeFeature::Plosive, PhonemeFeature::Velar, PhonemeFeature::Voiceless]).with_ipa("k"),
        WeightedPhoneme::new("s", 2).with_features(vec![PhonemeFeature::Fricative, PhonemeFeature::Alveolar, PhonemeFeature::Voiceless]).with_ipa("s"),
        WeightedPhoneme::new("m", 1).with_features(vec![PhonemeFeature::Nasal, PhonemeFeature::Labial, PhonemeFeature::Voiced]).with_ipa("m"),
    ];

    let templates = vec![
        SyllableTemplate::new(
            "CV",
            vec![
                Slot::required(PhonemeClass::OnsetConsonant),
                Slot::required(PhonemeClass::Vowel),
            ],
            2,
        ),
        SyllableTemplate::new(
            "CVC",
            vec![
                Slot::required(PhonemeClass::OnsetConsonant),
                Slot::required(PhonemeClass::Vowel),
                Slot::optional(PhonemeClass::CodaConsonant, 0.85),
            ],
            4,
        ),
        SyllableTemplate::new(
            "CCVC",
            vec![
                Slot::required(PhonemeClass::Cluster),
                Slot::required(PhonemeClass::Vowel),
                Slot::optional(PhonemeClass::CodaConsonant, 0.7),
            ],
            3,
        ),
    ];

    let phonology = Phonology::new(
        vowels,
        onset_consonants,
        clusters,
        coda_consonants,
        templates,
        PhonotacticConstraints::new(true),
    );

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

    let sound_changes = vec![
        SoundChange::new(
            "aspirate velar plosives",
            "k",
            vec!["kh"],
            0.9,
            Environment::anywhere(),
        ),
        SoundChange::new(
            "soften alveolar fricatives before vowels",
            "s",
            vec!["sh"],
            0.65,
            Environment::between(Matcher::Any, Matcher::Vowel),
        ),
        SoundChange::new(
            "palatalize stops before front vowels",
            "t",
            vec!["ts"],
            0.4,
            Environment::between(Matcher::Any, Matcher::HasFeature(PhonemeFeature::Front)),
        ),
        SoundChange::new(
            "brighten vowels between consonants",
            "a",
            vec!["ae"],
            0.55,
            Environment::between(Matcher::Consonant, Matcher::Consonant),
        ),
    ];

    let mut semantic_mapper = SemanticMapper::new();
    for gloss in DEMO_GLOSSES {
        let concept = Concept::new(*gloss, *gloss);
        let id = concept.id.clone();
        semantic_mapper.add_concept(concept);
        semantic_mapper.map_gloss(*gloss, id);
    }

    LanguageBlueprint::new("Concilium", phonology, grammar, sound_changes, semantic_mapper)
}
