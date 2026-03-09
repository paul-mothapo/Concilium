use crate::Language;
use crate::corpus::CorpusLoadReport;

pub fn render_english_to_concilium(language: &Language, corpus: &CorpusLoadReport) -> String {
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

    if !corpus.sentences.is_empty() {
        output.push_str(
            "\n## English Sentences to Concilium\n\n| English Sentence | Concilium Sentence | Pronunciation |\n| --- | --- | --- |\n",
        );

        for sentence in &corpus.sentences {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                sentence,
                language.translate_text(sentence),
                language.pronunciation_for_text(sentence)
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::render_english_to_concilium;
    use crate::corpus::CorpusLoadReport;
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
        let corpus = CorpusLoadReport {
            files: Vec::new(),
            glosses: vec!["you".to_owned()],
            sentences: vec!["You.".to_owned()],
        };

        let markdown = render_english_to_concilium(&language, &corpus);

        assert!(markdown.contains("# English to Concilium"));
        assert!(markdown.contains("| you | khi | [kh-ee] |"));
        assert!(markdown.contains("## English Sentences to Concilium"));
    }
}
