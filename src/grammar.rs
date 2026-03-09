use crate::form::WordForm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WordOrder {
    SOV,
    SVO,
    VSO,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grammar {
    pub word_order: WordOrder,
    pub plural_suffix: Option<Vec<String>>,
    pub past_prefix: Option<Vec<String>>,
}

impl Grammar {
    pub fn new(
        word_order: WordOrder,
        plural_suffix: Option<Vec<&str>>,
        past_prefix: Option<Vec<&str>>,
    ) -> Self {
        Self {
            word_order,
            plural_suffix: plural_suffix
                .map(|symbols| symbols.into_iter().map(str::to_owned).collect()),
            past_prefix: past_prefix
                .map(|symbols| symbols.into_iter().map(str::to_owned).collect()),
        }
    }

    pub fn pluralize(&self, word: &WordForm) -> WordForm {
        match &self.plural_suffix {
            Some(suffix) => word.with_suffix(suffix.clone()),
            None => word.clone(),
        }
    }

    pub fn make_past(&self, word: &WordForm) -> WordForm {
        match &self.past_prefix {
            Some(prefix) => word.with_prefix(prefix.clone()),
            None => word.clone(),
        }
    }

    pub fn realize_clause(&self, clause: &Clause) -> Vec<WordForm> {
        let subject = clause.subject.clone();
        let object = if clause.object_is_plural {
            self.pluralize(&clause.object)
        } else {
            clause.object.clone()
        };
        let verb = if clause.verb_is_past {
            self.make_past(&clause.verb)
        } else {
            clause.verb.clone()
        };

        match self.word_order {
            WordOrder::SOV => vec![subject, object, verb],
            WordOrder::SVO => vec![subject, verb, object],
            WordOrder::VSO => vec![verb, subject, object],
        }
    }

    pub fn render_clause(&self, clause: &Clause) -> String {
        self.realize_clause(clause)
            .into_iter()
            .map(|word| word.text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clause {
    pub subject: WordForm,
    pub object: WordForm,
    pub verb: WordForm,
    pub object_is_plural: bool,
    pub verb_is_past: bool,
}

impl Clause {
    pub fn new(subject: WordForm, object: WordForm, verb: WordForm) -> Self {
        Self {
            subject,
            object,
            verb,
            object_is_plural: false,
            verb_is_past: false,
        }
    }

    pub fn with_plural_object(mut self) -> Self {
        self.object_is_plural = true;
        self
    }

    pub fn with_past_verb(mut self) -> Self {
        self.verb_is_past = true;
        self
    }
}
