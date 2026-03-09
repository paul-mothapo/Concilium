use std::collections::HashSet;

use crate::form::WordForm;
use crate::phonology::Phonology;
use crate::rng::Random;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lexeme {
    pub gloss: String,
    pub form: WordForm,
}

impl Lexeme {
    pub fn new(gloss: impl Into<String>, form: WordForm) -> Self {
        Self {
            gloss: gloss.into(),
            form,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WordGenerationConfig {
    pub min_syllables: usize,
    pub max_syllables: usize,
}

impl WordGenerationConfig {
    pub fn new(min_syllables: usize, max_syllables: usize) -> Self {
        Self {
            min_syllables,
            max_syllables,
        }
    }
}

pub struct LexiconGenerator<'a> {
    phonology: &'a Phonology,
}

impl<'a> LexiconGenerator<'a> {
    pub fn new(phonology: &'a Phonology) -> Self {
        Self { phonology }
    }

    pub fn generate_word(&self, config: WordGenerationConfig, rng: &mut Random) -> WordForm {
        for _ in 0..256 {
            let syllable_count = if config.min_syllables == config.max_syllables {
                config.min_syllables
            } else {
                rng.range_usize(config.min_syllables..(config.max_syllables + 1))
            };

            let mut phonemes = Vec::new();
            for _ in 0..syllable_count {
                let template = self
                    .phonology
                    .sample_template(rng)
                    .expect("at least one syllable template is required");

                for slot in &template.slots {
                    if let Some(probability) = slot.optional_probability {
                        if !rng.coin(probability) {
                            continue;
                        }
                    }

                    if let Some(symbol) = self.phonology.sample_symbol(slot.class, rng) {
                        phonemes.push(symbol);
                    }
                }
            }

            let candidate = WordForm::new(phonemes);
            if self.phonology.allows(&candidate) {
                return candidate;
            }
        }

        panic!("failed to generate a valid word within the attempt limit");
    }

    pub fn generate_lexicon(
        &self,
        glosses: &[&str],
        config: WordGenerationConfig,
        rng: &mut Random,
    ) -> Vec<Lexeme> {
        let mut used_forms = HashSet::new();
        let mut lexicon = Vec::with_capacity(glosses.len());

        for gloss in glosses {
            let form = loop {
                let candidate = self.generate_word(config, rng);
                if used_forms.insert(candidate.text()) {
                    break candidate;
                }
            };

            lexicon.push(Lexeme::new(*gloss, form));
        }

        lexicon
    }
}
