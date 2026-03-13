use crate::form::WordForm;
use crate::phonology::Phonology;
use crate::rng::Random;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Matcher {
    Any,
    Start,
    End,
    Exact(String),
    OneOf(Vec<String>),
    Vowel,
    Consonant,
    HasFeature(crate::phonology::PhonemeFeature),
    HasFeatures(Vec<crate::phonology::PhonemeFeature>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Environment {
    pub left: Matcher,
    pub right: Matcher,
}

impl Environment {
    pub fn anywhere() -> Self {
        Self {
            left: Matcher::Any,
            right: Matcher::Any,
        }
    }

    pub fn between(left: Matcher, right: Matcher) -> Self {
        Self { left, right }
    }

    fn matches(&self, index: usize, symbols: &[String], phonology: &Phonology) -> bool {
        let left = if index == 0 {
            None
        } else {
            symbols.get(index - 1).map(String::as_str)
        };
        let right = symbols.get(index + 1).map(String::as_str);

        self.left.matches(left, phonology) && self.right.matches(right, phonology)
    }
}

impl Matcher {
    fn matches(&self, symbol: Option<&str>, phonology: &Phonology) -> bool {
        match self {
            Self::Any => true,
            Self::Start => symbol.is_none(),
            Self::End => symbol.is_none(),
            Self::Exact(expected) => symbol.is_some_and(|actual| actual == expected),
            Self::OneOf(options) => {
                symbol.is_some_and(|actual| options.iter().any(|option| option == actual))
            }
            Self::Vowel => symbol.is_some_and(|actual| phonology.is_vowel_symbol(actual)),
            Self::Consonant => symbol.is_some_and(|actual| phonology.is_consonant_symbol(actual)),
            Self::HasFeature(feature) => symbol.is_some_and(|actual| {
                phonology.find_phoneme(actual).map_or(false, |p| p.features.has(*feature))
            }),
            Self::HasFeatures(features) => symbol.is_some_and(|actual| {
                phonology.find_phoneme(actual).map_or(false, |p| {
                    features.iter().all(|f| p.features.has(*f))
                })
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundChange {
    pub name: String,
    pub from: String,
    pub to: Vec<String>,
    pub probability: f32,
    pub environment: Environment,
}

impl SoundChange {
    pub fn new(
        name: impl Into<String>,
        from: impl Into<String>,
        to: Vec<&str>,
        probability: f32,
        environment: Environment,
    ) -> Self {
        Self {
            name: name.into(),
            from: from.into(),
            to: to.into_iter().map(str::to_owned).collect(),
            probability,
            environment,
        }
    }

    pub fn apply(&self, form: &WordForm, phonology: &Phonology, rng: &mut Random) -> WordForm {
        let source = form.phonemes();
        let mut result = Vec::with_capacity(source.len());

        for (index, symbol) in source.iter().enumerate() {
            if symbol == &self.from
                && self.environment.matches(index, source, phonology)
                && rng.coin(self.probability)
            {
                result.extend(self.to.iter().cloned());
            } else {
                result.push(symbol.clone());
            }
        }

        WordForm::new(result)
    }

    pub fn apply_sequence(
        changes: &[SoundChange],
        form: &WordForm,
        phonology: &Phonology,
        rng: &mut Random,
    ) -> WordForm {
        changes.iter().fold(form.clone(), |current, change| {
            change.apply(&current, phonology, rng)
        })
    }
}
