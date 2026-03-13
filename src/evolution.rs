use std::collections::BTreeSet;

use crate::form::WordForm;
use crate::grammar::{Grammar, SyntaxNode, PhraseCategory, FeatureValue};
use crate::lexicon::{Lexeme, LexiconGenerator, WordGenerationConfig};
use crate::mutation::SoundChange;
use crate::phonology::{Phonology, WeightedPhoneme};
use crate::rng::Random;
use crate::semantics::{SemanticMapper, ConceptId};

#[derive(Clone, Debug)]
pub struct LanguageBlueprint {
    pub name: String,
    pub phonology: Phonology,
    pub grammar: Grammar,
    pub sound_changes: Vec<SoundChange>,
    pub semantic_mapper: SemanticMapper,
}

impl LanguageBlueprint {
    pub fn new(
        name: impl Into<String>,
        phonology: Phonology,
        grammar: Grammar,
        sound_changes: Vec<SoundChange>,
        semantic_mapper: SemanticMapper,
    ) -> Self {
        Self {
            name: name.into(),
            phonology,
            grammar,
            sound_changes,
            semantic_mapper,
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
    pub semantic_mapper: SemanticMapper,
}

#[derive(Clone, Debug)]
pub enum GlossSyntaxNode {
    Leaf(String),
    Branch {
        category: PhraseCategory,
        children: Vec<GlossSyntaxNode>,
    },
}

impl GlossSyntaxNode {
    pub fn leaf(gloss: impl Into<String>) -> Self {
        Self::Leaf(gloss.into())
    }

    pub fn branch(category: PhraseCategory, children: Vec<GlossSyntaxNode>) -> Self {
        Self::Branch { category, children }
    }
}

impl Language {
    pub fn lexeme_by_concept(&self, concept_id: &ConceptId) -> Option<&Lexeme> {
        self.lexicon.iter().find(|lexeme| lexeme.concept_id == *concept_id)
    }

    pub fn lexemes_for_gloss(&self, gloss: &str) -> Vec<&Lexeme> {
        let concept_ids = self.semantic_mapper.resolve_gloss(gloss);
        concept_ids.iter()
            .filter_map(|id| self.lexeme_by_concept(id))
            .collect()
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
        let mut features = Vec::new();
        if object_is_plural {
            features.push(FeatureValue::Plural); 
        }
        if verb_is_past {
            features.push(FeatureValue::Past);
        }

        let tree = GlossSyntaxNode::branch(
            PhraseCategory::Sentence,
            vec![
                GlossSyntaxNode::leaf(subject_gloss),
                GlossSyntaxNode::leaf(object_gloss),
                GlossSyntaxNode::leaf(verb_gloss),
            ],
        );

        self.render_tree_from_glosses_with_features(&tree, &features)
    }

    pub fn render_tree(&self, node: &SyntaxNode, features: &[FeatureValue]) -> String {
        self.grammar.render_node(node, features)
    }

    pub fn render_tree_from_glosses(&self, tree: &GlossSyntaxNode) -> Option<String> {
        self.render_tree_from_glosses_with_features(tree, &[])
    }

    pub fn render_tree_from_glosses_with_features(
        &self,
        tree: &GlossSyntaxNode,
        features: &[FeatureValue],
    ) -> Option<String> {
        let syntax_tree = self.resolve_gloss_tree(tree)?;
        Some(self.render_tree(&syntax_tree, features))
    }

    fn resolve_gloss_tree(&self, tree: &GlossSyntaxNode) -> Option<SyntaxNode> {
        match tree {
            GlossSyntaxNode::Leaf(gloss) => {
                let lexeme = self.lexemes_for_gloss(gloss).into_iter().next()?;
                Some(SyntaxNode::Leaf(lexeme.form.clone()))
            }
            GlossSyntaxNode::Branch { category, children } => {
                let mut syntax_children = Vec::new();
                for child in children {
                    syntax_children.push(self.resolve_gloss_tree(child)?);
                }
                Some(SyntaxNode::Branch {
                    category: *category,
                    children: syntax_children,
                })
            }
        }
    }

    pub fn translate_text(&self, text: &str) -> String {
        let mut translated = Vec::new();

        for token in tokenize_text(text) {
            match token {
                TextToken::Word(word) => {
                    if let Some(lexeme) = self.lexemes_for_gloss(&word).into_iter().next() {
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
                    if let Some(lexeme) = self.lexemes_for_gloss(&word).into_iter().next() {
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
        config: WordGenerationConfig,
        rng: &mut Random,
    ) -> Language {
        let generator = LexiconGenerator::new(&blueprint.phonology);
        
        // Collect all concepts from the mapper
        let concepts: Vec<ConceptId> = blueprint.semantic_mapper.concepts.keys().cloned().collect();
        
        let lexicon = generator
            .generate_lexicon(&concepts, config, rng)
            .into_iter()
            .map(|lexeme| {
                let form = SoundChange::apply_sequence(
                    &blueprint.sound_changes,
                    &lexeme.form,
                    &blueprint.phonology,
                    rng,
                );
                Lexeme::new(lexeme.concept_id, form)
            })
            .collect();

        Language {
            name: blueprint.name.clone(),
            phonology: derive_phonology(&blueprint.phonology, &blueprint.sound_changes),
            grammar: blueprint.grammar.clone(),
            lexicon,
            sound_changes: blueprint.sound_changes.clone(),
            semantic_mapper: blueprint.semantic_mapper.clone(),
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
