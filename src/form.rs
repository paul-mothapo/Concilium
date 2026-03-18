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

    pub fn pronunciation(&self) -> String {
        self.phonemes
            .iter()
            .map(|phoneme| approximate_pronunciation(phoneme))
            .collect::<Vec<_>>()
            .join("-")
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

    pub fn to_ipa_string(&self, phonology: &crate::phonology::Phonology) -> String {
        self.phonemes
            .iter()
            .map(|symbol| {
                phonology
                    .find_phoneme(symbol)
                    .and_then(|p| p.ipa.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or(symbol)
            })
            .collect::<String>()
    }
}

impl Display for WordForm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

fn approximate_pronunciation(phoneme: &str) -> String {
    match phoneme {
        "a" => "ah".to_owned(),
        "ae" => "eye".to_owned(),
        "e" => "eh".to_owned(),
        "i" => "ee".to_owned(),
        "o" => "oh".to_owned(),
        "u" => "oo".to_owned(),
        "kh" => "kh".to_owned(),
        "sh" => "sh".to_owned(),
        "zh" => "zh".to_owned(),
        "dr" => "dr".to_owned(),
        "kr" => "kr".to_owned(),
        "tl" => "tl".to_owned(),
        other => other.to_owned(),
    }
}
