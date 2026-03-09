use crate::evolution::LanguageBlueprint;
use crate::grammar::{Grammar, WordOrder};
use crate::lexicon::WordGenerationConfig;
use crate::mutation::{Environment, Matcher, SoundChange};
use crate::phonology::{
    PhonemeClass, Phonology, PhonotacticConstraints, Slot, SyllableTemplate, WeightedPhoneme,
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
        WeightedPhoneme::new("a", 5),
        WeightedPhoneme::new("e", 4),
        WeightedPhoneme::new("i", 3),
        WeightedPhoneme::new("o", 3),
        WeightedPhoneme::new("u", 2),
    ];

    let onset_consonants = vec![
        WeightedPhoneme::new("l", 4),
        WeightedPhoneme::new("m", 4),
        WeightedPhoneme::new("n", 4),
        WeightedPhoneme::new("r", 4),
        WeightedPhoneme::new("k", 3),
        WeightedPhoneme::new("t", 3),
        WeightedPhoneme::new("g", 2),
        WeightedPhoneme::new("d", 2),
        WeightedPhoneme::new("s", 3),
        WeightedPhoneme::new("v", 2),
    ];

    let clusters = vec![
        WeightedPhoneme::new("dr", 2),
        WeightedPhoneme::new("kr", 2),
        WeightedPhoneme::new("sh", 2),
        WeightedPhoneme::new("tl", 1),
    ];

    let coda_consonants = vec![
        WeightedPhoneme::new("l", 2),
        WeightedPhoneme::new("n", 3),
        WeightedPhoneme::new("r", 3),
        WeightedPhoneme::new("k", 2),
        WeightedPhoneme::new("s", 2),
        WeightedPhoneme::new("m", 1),
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

    let grammar = Grammar::new(WordOrder::SOV, Some(vec!["e", "n"]), Some(vec!["ka"]));

    let sound_changes = vec![
        SoundChange::new(
            "aspirate velars",
            "k",
            vec!["kh"],
            0.9,
            Environment::anywhere(),
        ),
        SoundChange::new(
            "soften s before vowels",
            "s",
            vec!["sh"],
            0.65,
            Environment::between(Matcher::Any, Matcher::Vowel),
        ),
        SoundChange::new(
            "brighten a",
            "a",
            vec!["ae"],
            0.55,
            Environment::between(Matcher::Consonant, Matcher::Consonant),
        ),
    ];

    LanguageBlueprint::new("Concilium", phonology, grammar, sound_changes)
}
