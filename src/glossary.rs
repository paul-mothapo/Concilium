use crate::Language;

pub fn render_english_to_concilium(language: &Language) -> String {
    let mut rows = language.lexicon.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.gloss.cmp(&right.gloss));

    let mut output = String::from(
        "# English to Concilium\n\n| English | Concilium | Pronunciation |\n| --- | --- | --- |\n",
    );

    for lexeme in rows {
        output.push_str(&format!(
            "| {} | {} | [{}] |\n",
            lexeme.gloss,
            lexeme.form.text(),
            lexeme.form.pronunciation()
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::render_english_to_concilium;
    use crate::lexicon::Lexeme;
    use crate::{
        Grammar, Language, Phonology, PhonotacticConstraints, WeightedPhoneme, WordForm, WordOrder,
    };

    #[test]
    fn renders_markdown_glossary_with_pronunciation() {
        let language = Language {
            name: "Concilium".to_owned(),
            phonology: Phonology::new(
                vec![WeightedPhoneme::new("i", 1)],
                vec![WeightedPhoneme::new("kh", 1)],
                vec![],
                vec![],
                vec![],
                PhonotacticConstraints::new(false),
            ),
            grammar: Grammar::new(WordOrder::SOV, None, None),
            lexicon: vec![Lexeme::new("you", WordForm::new(["kh", "i"]))],
            sound_changes: Vec::new(),
        };

        let markdown = render_english_to_concilium(&language);

        assert!(markdown.contains("# English to Concilium"));
        assert!(markdown.contains("| you | khi | [kh-ee] |"));
    }
}
