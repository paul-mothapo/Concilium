use crate::form::WordForm;
use crate::rng::Random;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedPhoneme {
    pub symbol: String,
    pub weight: u32,
}

impl WeightedPhoneme {
    pub fn new(symbol: impl Into<String>, weight: u32) -> Self {
        Self {
            symbol: symbol.into(),
            weight,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhonemeClass {
    OnsetConsonant,
    Cluster,
    Vowel,
    CodaConsonant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub class: PhonemeClass,
    pub optional_probability: Option<f32>,
}

impl Slot {
    pub fn required(class: PhonemeClass) -> Self {
        Self {
            class,
            optional_probability: None,
        }
    }

    pub fn optional(class: PhonemeClass, probability: f32) -> Self {
        Self {
            class,
            optional_probability: Some(probability),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SyllableTemplate {
    pub name: String,
    pub slots: Vec<Slot>,
    pub weight: u32,
}

impl SyllableTemplate {
    pub fn new(name: impl Into<String>, slots: Vec<Slot>, weight: u32) -> Self {
        Self {
            name: name.into(),
            slots,
            weight,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhonotacticConstraints {
    pub disallow_initial_vowel: bool,
}

impl PhonotacticConstraints {
    pub fn new(disallow_initial_vowel: bool) -> Self {
        Self {
            disallow_initial_vowel,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Phonology {
    pub vowels: Vec<WeightedPhoneme>,
    pub onset_consonants: Vec<WeightedPhoneme>,
    pub clusters: Vec<WeightedPhoneme>,
    pub coda_consonants: Vec<WeightedPhoneme>,
    pub templates: Vec<SyllableTemplate>,
    pub constraints: PhonotacticConstraints,
}

impl Phonology {
    pub fn new(
        vowels: Vec<WeightedPhoneme>,
        onset_consonants: Vec<WeightedPhoneme>,
        clusters: Vec<WeightedPhoneme>,
        coda_consonants: Vec<WeightedPhoneme>,
        templates: Vec<SyllableTemplate>,
        constraints: PhonotacticConstraints,
    ) -> Self {
        Self {
            vowels,
            onset_consonants,
            clusters,
            coda_consonants,
            templates,
            constraints,
        }
    }

    pub fn sample_template<'a>(&'a self, rng: &mut Random) -> Option<&'a SyllableTemplate> {
        let weights = self
            .templates
            .iter()
            .map(|template| template.weight)
            .collect::<Vec<_>>();
        rng.weighted_index(&weights)
            .and_then(|index| self.templates.get(index))
    }

    pub fn sample_symbol(&self, class: PhonemeClass, rng: &mut Random) -> Option<String> {
        let inventory = match class {
            PhonemeClass::OnsetConsonant => &self.onset_consonants,
            PhonemeClass::Cluster => &self.clusters,
            PhonemeClass::Vowel => &self.vowels,
            PhonemeClass::CodaConsonant => &self.coda_consonants,
        };

        let weights = inventory
            .iter()
            .map(|symbol| symbol.weight)
            .collect::<Vec<_>>();
        rng.weighted_index(&weights)
            .and_then(|index| inventory.get(index))
            .map(|phoneme| phoneme.symbol.clone())
    }

    pub fn allows(&self, form: &WordForm) -> bool {
        if !self.constraints.disallow_initial_vowel {
            return true;
        }

        form.phonemes()
            .first()
            .map(|symbol| !self.is_vowel_symbol(symbol))
            .unwrap_or(false)
    }

    pub fn is_vowel_symbol(&self, symbol: &str) -> bool {
        self.vowels.iter().any(|phoneme| phoneme.symbol == symbol)
            || symbol
                .chars()
                .any(|character| matches!(character, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
    }

    pub fn is_consonant_symbol(&self, symbol: &str) -> bool {
        !self.is_vowel_symbol(symbol)
    }
}
