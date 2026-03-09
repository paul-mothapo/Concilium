use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WordForm {
    phonemes: Vec<String>,
}

impl WordForm {
    pub fn new<I, S>(phonemes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            phonemes: phonemes.into_iter().map(Into::into).collect(),
        }
    }

    pub fn phonemes(&self) -> &[String] {
        &self.phonemes
    }

    pub fn text(&self) -> String {
        self.phonemes.concat()
    }

    pub fn with_prefix<I, S>(&self, prefix: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut phonemes = prefix.into_iter().map(Into::into).collect::<Vec<_>>();
        phonemes.extend(self.phonemes.clone());
        Self { phonemes }
    }

    pub fn with_suffix<I, S>(&self, suffix: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut phonemes = self.phonemes.clone();
        phonemes.extend(suffix.into_iter().map(Into::into));
        Self { phonemes }
    }
}

impl Display for WordForm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}
