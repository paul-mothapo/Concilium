use std::collections::BTreeSet;

use crate::form::WordForm;
use crate::grammar::{Clause, Grammar};
use crate::lexicon::{Lexeme, LexiconGenerator, WordGenerationConfig};
use crate::mutation::SoundChange;
use crate::phonology::{Phonology, WeightedPhoneme};
use crate::rng::Random;

#[derive(Clone, Debug)]
pub struct LanguageBlueprint {
    pub name: String,
    pub phonology: Phonology,
    pub grammar: Grammar,
    pub sound_changes: Vec<SoundChange>,
}

impl LanguageBlueprint {
    pub fn new(
        name: impl Into<String>,
        phonology: Phonology,
        grammar: Grammar,
        sound_changes: Vec<SoundChange>,
    ) -> Self {
        Self {
            name: name.into(),
            phonology,
            grammar,
            sound_changes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Language {
    pub name: String,
    pub phonology: Phonology,
    pub grammar: Grammar,
    pub lexicon: Vec<Lexeme>,
    pub sound_changes: Vec<SoundChange>,
}

impl Language {
    pub fn lexeme(&self, gloss: &str) -> Option<&Lexeme> {
        self.lexicon.iter().find(|lexeme| lexeme.gloss == gloss)
    }

    pub fn sample_words(&self, limit: usize) -> Vec<&WordForm> {
        self.lexicon
            .iter()
            .take(limit)
            .map(|lexeme| &lexeme.form)
            .collect()
    }

    pub fn render_clause_from_glosses(
        &self,
        subject_gloss: &str,
        object_gloss: &str,
        verb_gloss: &str,
        object_is_plural: bool,
        verb_is_past: bool,
    ) -> Option<String> {
        let subject = self.lexeme(subject_gloss)?.form.clone();
        let object = self.lexeme(object_gloss)?.form.clone();
        let verb = self.lexeme(verb_gloss)?.form.clone();

        let mut clause = Clause::new(subject, object, verb);
        if object_is_plural {
            clause = clause.with_plural_object();
        }
        if verb_is_past {
            clause = clause.with_past_verb();
        }

        Some(self.grammar.render_clause(&clause))
    }

    pub fn translate_text(&self, text: &str) -> String {
        let mut translated = Vec::new();

        for token in tokenize_text(text) {
            match token {
                TextToken::Word(word) => {
                    if let Some(lexeme) = self.lexeme(&word) {
                        translated.push(lexeme.form.text());
                    } else {
                        translated.push(word);
                    }
                }
                TextToken::Punctuation(mark) => translated.push(mark.to_string()),
            }
        }

        join_translated_tokens(&translated)
    }

    pub fn pronunciation_for_text(&self, text: &str) -> String {
        let mut pronunciation = Vec::new();

        for token in tokenize_text(text) {
            match token {
                TextToken::Word(word) => {
                    if let Some(lexeme) = self.lexeme(&word) {
                        pronunciation.push(format!("[{}]", lexeme.form.pronunciation()));
                    } else {
                        pronunciation.push(format!("[{}]", word));
                    }
                }
                TextToken::Punctuation(mark) => pronunciation.push(mark.to_string()),
            }
        }

        join_translated_tokens(&pronunciation)
    }

    pub fn inventory_snapshot(&self) -> Vec<String> {
        let mut symbols = BTreeSet::new();
        for inventory in [
            &self.phonology.vowels,
            &self.phonology.onset_consonants,
            &self.phonology.clusters,
            &self.phonology.coda_consonants,
        ] {
            for phoneme in inventory {
                symbols.insert(phoneme.symbol.clone());
            }
        }

        if symbols.is_empty() {
            for lexeme in &self.lexicon {
                for symbol in lexeme.form.phonemes() {
                    symbols.insert(symbol.clone());
                }
            }
        }

        symbols.into_iter().collect()
    }
}

#[derive(Default)]
pub struct LanguageEngine;

impl LanguageEngine {
    pub fn generate_language(
        &self,
        blueprint: &LanguageBlueprint,
        glosses: &[&str],
        config: WordGenerationConfig,
        rng: &mut Random,
    ) -> Language {
        let generator = LexiconGenerator::new(&blueprint.phonology);
        let lexicon = generator
            .generate_lexicon(glosses, config, rng)
            .into_iter()
            .map(|lexeme| {
                let form = SoundChange::apply_sequence(
                    &blueprint.sound_changes,
                    &lexeme.form,
                    &blueprint.phonology,
                    rng,
                );
                Lexeme::new(lexeme.gloss, form)
            })
            .collect();

        Language {
            name: blueprint.name.clone(),
            phonology: derive_phonology(&blueprint.phonology, &blueprint.sound_changes),
            grammar: blueprint.grammar.clone(),
            lexicon,
            sound_changes: blueprint.sound_changes.clone(),
        }
    }
}

fn derive_phonology(phonology: &Phonology, sound_changes: &[SoundChange]) -> Phonology {
    Phonology {
        vowels: derive_inventory(&phonology.vowels, sound_changes),
        onset_consonants: derive_inventory(&phonology.onset_consonants, sound_changes),
        clusters: derive_inventory(&phonology.clusters, sound_changes),
        coda_consonants: derive_inventory(&phonology.coda_consonants, sound_changes),
        templates: phonology.templates.clone(),
        constraints: phonology.constraints.clone(),
    }
}

fn derive_inventory(
    inventory: &[WeightedPhoneme],
    sound_changes: &[SoundChange],
) -> Vec<WeightedPhoneme> {
    let mut derived = inventory.to_vec();

    for change in sound_changes {
        for phoneme in &mut derived {
            if phoneme.symbol == change.from && change.to.len() == 1 {
                phoneme.symbol = change.to[0].clone();
            }
        }
    }

    let mut unique = BTreeSet::new();
    derived.retain(|phoneme| unique.insert(phoneme.symbol.clone()));
    derived
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TextToken {
    Word(String),
    Punctuation(char),
}

fn tokenize_text(text: &str) -> Vec<TextToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for character in text.chars() {
        if character.is_ascii_alphabetic() || character == '\'' {
            current.push(character.to_ascii_lowercase());
            continue;
        }

        if !current.is_empty() {
            tokens.push(TextToken::Word(std::mem::take(&mut current)));
        }

        if !character.is_whitespace() {
            tokens.push(TextToken::Punctuation(character));
        }
    }

    if !current.is_empty() {
        tokens.push(TextToken::Word(current));
    }

    tokens
}

fn join_translated_tokens(tokens: &[String]) -> String {
    let mut output = String::new();

    for token in tokens {
        let is_punctuation = token.len() == 1
            && token
                .chars()
                .next()
                .is_some_and(|character| !character.is_ascii_alphanumeric());

        if is_punctuation {
            output.push_str(token);
        } else {
            if !output.is_empty() {
                output.push(' ');
            }
            output.push_str(token);
        }
    }

    output
}
