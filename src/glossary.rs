use crate::Language;
use crate::corpus::CorpusLoadReport;

pub fn render_lexicon_markdown(language: &Language) -> String {
    let mut rows = language.lexicon.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.concept_id.0.cmp(&right.concept_id.0));

    let mut output = String::from(
        "# English to Concilium: Words\n\n| English | Concilium | Pronunciation |\n| --- | --- | --- |\n",
    );

    for lexeme in rows {
        output.push_str(&format!(
            "| {} | {} | [{}] |\n",
            lexeme.concept_id.0,
            lexeme.form.text(),
            lexeme.form.pronunciation()
        ));
    }

    output
}

pub fn render_sentences_markdown(language: &Language, corpus: &CorpusLoadReport) -> String {
    let mut output = String::from(
        "# English to Concilium: Sentences\n\n| English Sentence | Concilium Sentence | Pronunciation |\n| --- | --- | --- |\n",
    );

    for sentence in &corpus.sentences {
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            sentence,
            language.translate_text(sentence),
            language.pronunciation_for_text(sentence)
        ));
    }

    output
}

pub fn render_english_to_concilium(language: &Language, corpus: &CorpusLoadReport) -> String {
    let mut output = render_lexicon_markdown(language);

    if !corpus.sentences.is_empty() {
        output.push_str("\n");
        output.push_str(&render_sentences_markdown(language, corpus));
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
        let mut semantic_mapper = crate::SemanticMapper::new();
        let concept = crate::Concept::new("you", "you");
        let id = concept.id.clone();
        semantic_mapper.add_concept(concept);
        semantic_mapper.map_gloss("you", id.clone());

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
            grammar: Grammar::new(WordOrder::SOV, crate::grammar::MorphologyEngine::default()),
            lexicon: vec![Lexeme::new(id, WordForm::new(["kh", "i"]))],
            sound_changes: Vec::new(),
            semantic_mapper,
        };
        let corpus = CorpusLoadReport {
            files: Vec::new(),
            glosses: vec!["you".to_owned()],
            sentences: vec!["You.".to_owned()],
            api_sources: Vec::new(),
        };

        let markdown = render_english_to_concilium(&language, &corpus);

        assert!(markdown.contains("# English to Concilium: Words"));
        assert!(markdown.contains("| you | khi | [kh-ee] |"));
        assert!(markdown.contains("# English to Concilium: Sentences"));
    }
}
