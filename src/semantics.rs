use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ConceptId(pub String);

impl ConceptId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Concept {
    pub id: ConceptId,
    pub canonical_name: String,
}

impl Concept {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: ConceptId::new(id),
            canonical_name: name.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SemanticMapper {
    /// Maps English glosses to one or more Concept IDs (Polysemy/Synonymy)
    pub english_to_concepts: HashMap<String, Vec<ConceptId>>,
    /// Registry of all known concepts
    pub concepts: HashMap<ConceptId, Concept>,
}

impl SemanticMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_concept(&mut self, concept: Concept) {
        self.concepts.insert(concept.id.clone(), concept);
    }

    pub fn map_gloss(&mut self, gloss: impl Into<String>, concept_id: ConceptId) {
        self.english_to_concepts
            .entry(gloss.into())
            .or_default()
            .push(concept_id);
    }

    pub fn resolve_gloss(&self, gloss: &str) -> Vec<ConceptId> {
        self.english_to_concepts
            .get(gloss)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_concept(&self, id: &ConceptId) -> Option<&Concept> {
        self.concepts.get(id)
    }
}
