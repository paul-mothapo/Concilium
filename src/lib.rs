pub mod corpus;
pub mod evolution;
pub mod form;
pub mod glossary;
pub mod grammar;
pub mod lexicon;
pub mod mutation;
pub mod phonology;
pub mod presets;
pub mod voice;
pub mod semantics;

mod rng;

#[cfg(test)]
mod tests;

pub use evolution::{Language, LanguageBlueprint, LanguageEngine};
pub use form::WordForm;
pub use grammar::{Grammar, WordOrder, SyntaxNode, PhraseCategory, FeatureValue, MorphologyEngine, ParadigmRule};
pub use lexicon::{Lexeme, LexiconGenerator, WordGenerationConfig};
pub use mutation::{Environment, Matcher, SoundChange};
pub use phonology::{
    PhonemeClass, Phonology, PhonotacticConstraints, Slot, SyllableTemplate, WeightedPhoneme, PhonemeFeature,
};
pub use semantics::{Concept, ConceptId, SemanticMapper};
