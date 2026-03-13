use crate::form::WordForm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WordOrder {
    SOV,
    SVO,
    VSO,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhraseCategory {
    Sentence,
    NounPhrase,
    VerbPhrase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyntaxNode {
    Leaf(WordForm),
    Branch {
        category: PhraseCategory,
        children: Vec<SyntaxNode>,
    },
}

impl SyntaxNode {
    pub fn leaf(word: WordForm) -> Self {
        Self::Leaf(word)
    }

    pub fn branch(category: PhraseCategory, children: Vec<SyntaxNode>) -> Self {
        Self::Branch { category, children }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FeatureType {
    Number,
    Tense,
    Person,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum FeatureValue {
    Singular,
    Plural,
    Present,
    Past,
    Future,
    First,
    Second,
    Third,
}

impl FeatureValue {
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Singular | Self::Plural)
    }

    pub fn is_tense(&self) -> bool {
        matches!(self, Self::Present | Self::Past | Self::Future)
    }

    pub fn is_person(&self) -> bool {
        matches!(self, Self::First | Self::Second | Self::Third)
    }
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParadigmRule {
    pub feature: FeatureValue,
    pub prefix: Option<Vec<String>>,
    pub suffix: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MorphologyEngine {
    pub rules: Vec<ParadigmRule>,
}

impl MorphologyEngine {
    pub fn apply(&self, word: &WordForm, features: &[FeatureValue]) -> WordForm {
        let mut result = word.clone();
        for feature in features {
            if let Some(rule) = self.rules.iter().find(|r| r.feature == *feature) {
                if let Some(prefix) = &rule.prefix {
                    result = result.with_prefix(prefix.clone());
                }
                if let Some(suffix) = &rule.suffix {
                    result = result.with_suffix(suffix.clone());
                }
            }
        }
        result
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grammar {
    pub word_order: WordOrder,
    pub morphology: MorphologyEngine,
}

impl Grammar {
    pub fn new(
        word_order: WordOrder,
        morphology: MorphologyEngine,
    ) -> Self {
        Self {
            word_order,
            morphology,
        }
    }

    pub fn realize_node(&self, node: &SyntaxNode, features: &[FeatureValue]) -> Vec<WordForm> {
        match node {
            SyntaxNode::Leaf(word) => {
                let inflected = self.morphology.apply(word, features);
                vec![inflected]
            }
            SyntaxNode::Branch { category, children } => {
                let realized_children: Vec<Vec<WordForm>> = match category {
                    PhraseCategory::Sentence if children.len() == 3 => {
                        // S O V feature percolation
                        children
                            .iter()
                            .enumerate()
                            .map(|(i, child)| {
                                let filtered_features: Vec<FeatureValue> = features
                                    .iter()
                                    .filter(|f| match i {
                                        0 => f.is_person(), // Subject gets Person
                                        1 => f.is_number(), // Object gets Number
                                        2 => f.is_tense(),  // Verb gets Tense
                                        _ => false,
                                    })
                                    .cloned()
                                    .collect();
                                self.realize_node(child, &filtered_features)
                            })
                            .collect()
                    }
                    _ => {
                        // For noun phrases or other unknown structures, pass all features down
                        children
                            .iter()
                            .map(|child| self.realize_node(child, features))
                            .collect()
                    }
                };

                match category {
                    PhraseCategory::Sentence => self.order_sentence(realized_children),
                    _ => realized_children.into_iter().flatten().collect(),
                }
            }

        }
    }

    fn order_sentence(&self, children: Vec<Vec<WordForm>>) -> Vec<WordForm> {
        // For now we add simple mapping: children[0]=S, children[1]=O, children[2]=V
        if children.len() != 3 {
            return children.into_iter().flatten().collect();
        }

        let s = children[0].clone();
        let o = children[1].clone();
        let v = children[2].clone();

        match self.word_order {
            WordOrder::SOV => [s, o, v].concat(),
            WordOrder::SVO => [s, v, o].concat(),
            WordOrder::VSO => [v, s, o].concat(),
        }
    }

    pub fn render_node(&self, node: &SyntaxNode, features: &[FeatureValue]) -> String {
        self.realize_node(node, features)
            .into_iter()
            .map(|word| word.text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
