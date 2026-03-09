pub mod corpus;
pub mod evolution;
pub mod form;
pub mod glossary;
pub mod grammar;
pub mod lexicon;
pub mod mutation;
pub mod phonology;
pub mod presets;

mod rng;

#[cfg(test)]
mod tests;

pub use evolution::{Language, LanguageBlueprint, LanguageEngine};
pub use form::WordForm;
pub use grammar::{Clause, Grammar, WordOrder};
pub use lexicon::{Lexeme, LexiconGenerator, WordGenerationConfig};
pub use mutation::{Environment, Matcher, SoundChange};
pub use phonology::{
    PhonemeClass, Phonology, PhonotacticConstraints, Slot, SyllableTemplate, WeightedPhoneme,
};
