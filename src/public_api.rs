use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::corpus::CorpusLoadReport;

const RANDOM_WORD_API: &str = "https://random-word-api.herokuapp.com/word";
const DUMMY_JSON_QUOTES_API: &str = "https://dummyjson.com/quotes/random";
const BACON_IPSUM_API: &str = "https://baconipsum.com/api/";

pub fn fetch_public_api_corpus(context_window: usize) -> Result<CorpusLoadReport, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("failed to build HTTP client: {error}"))?;

    let word_target = context_window;
    let sentence_target = (context_window / 25).clamp(10, 400);

    let words_result = fetch_random_words(&client, word_target);
    let passages_result = fetch_public_passages(&client, sentence_target);

    let mut api_sources = Vec::new();
    let mut errors = Vec::new();

    let words = match words_result {
        Ok(words) if !words.is_empty() => {
            api_sources.push(format!(
                "Random Word API generated {word_target} words"
            ));
            words
        }
        Ok(_) => Vec::new(),
        Err(error) => {
            errors.push(error);
            Vec::new()
        }
    };

    let (sentences, passage_source) = match passages_result {
        Ok((sentences, source)) => (sentences, Some(source)),
        Err(error) => {
            errors.push(error);
            (Vec::new(), None)
        }
    };

    if let Some(source) = passage_source {
        api_sources.push(source);
    }

    if words.is_empty() && sentences.is_empty() {
        return Err(errors.join(" | "));
    }

    Ok(CorpusLoadReport {
        files: Vec::new(),
        glosses: words,
        sentences,
        api_sources,
    })
}

fn fetch_random_words(client: &Client, target: usize) -> Result<Vec<String>, String> {
    let limit = target.clamp(1, 1500);
    
    let response = client
        .get(RANDOM_WORD_API)
        .query(&[("number", &limit.to_string())])
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Random Word API request failed: {error}"))?;

    let payload = response.json::<Vec<String>>().map_err(|error| {
        format!("Random Word API response parse failed: {error}")
    })?;

    Ok(payload
        .into_iter()
        .filter(|word| {
            word.chars()
                .all(|character| character.is_ascii_alphabetic())
        })
        .collect())
}



fn fetch_dummyjson_passages(client: &Client, target: usize) -> Result<Vec<String>, String> {
    let mut sentences = Vec::new();
    let mut remaining = target;

    while remaining > 0 {
        let batch_size = remaining.min(50);
        let response = client
            .get(format!("{DUMMY_JSON_QUOTES_API}/{batch_size}"))
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(|error| format!("DummyJSON request failed: {error}"))?;

        let payload = response.json::<Vec<DummyJsonQuoteData>>().map_err(|error| {
            format!("DummyJSON response parse failed: {error}")
        })?;

        let mut batch = payload
            .into_iter()
            .map(|data| data.quote)
            .filter(|quote| !quote.trim().is_empty())
            .filter(|quote| looks_like_english_sentence(quote))
            .collect::<Vec<String>>();

        if batch.is_empty() {
            break;
        }

        remaining = remaining.saturating_sub(batch.len());
        sentences.append(&mut batch);

        if sentences.len() >= target {
            break;
        }
    }

    if sentences.is_empty() {
        return Err("DummyJSON returned no passages".to_owned());
    }

    Ok(sentences.into_iter().take(target).collect())
}

fn fetch_public_passages(client: &Client, target: usize) -> Result<(Vec<String>, String), String> {
    let mut errors = Vec::new();

    match fetch_dummyjson_passages(client, target) {
        Ok(sentences) if !sentences.is_empty() => {
            return Ok((
                sentences,
                format!("DummyJSON random quotes toward {target} passages"),
            ));
        }
        Ok(_) => errors.push("DummyJSON returned no passages".to_owned()),
        Err(error) => errors.push(error),
    }

    match fetch_bacon_ipsum_passages(client, target) {
        Ok(sentences) if !sentences.is_empty() => {
            return Ok((
                sentences,
                format!("Bacon Ipsum batched paragraphs toward {target} passages"),
            ));
        }
        Ok(_) => errors.push("Bacon Ipsum returned no passages".to_owned()),
        Err(error) => errors.push(error),
    }

    Err(errors.join(" | "))
}

fn fetch_bacon_ipsum_passages(client: &Client, target: usize) -> Result<Vec<String>, String> {
    let paragraph_count = target.clamp(1, 50);
    let response = client
        .get(BACON_IPSUM_API)
        .query(&[
            ("type", "meat-and-filler".to_owned()),
            ("paras", paragraph_count.to_string()),
            ("format", "json".to_owned()),
        ])
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Bacon Ipsum request failed: {error}"))?;

    response
        .json::<Vec<String>>()
        .map(|paragraphs| {
            paragraphs
                .into_iter()
                .filter(|text| looks_like_english_sentence(text))
                .take(target)
                .collect()
        })
        .map_err(|error| format!("Bacon Ipsum response parse failed: {error}"))
}

#[derive(Clone, Debug, Deserialize)]
struct DummyJsonQuoteData {
    quote: String,
}


fn looks_like_english_sentence(text: &str) -> bool {
    let mut has_letter = false;

    for character in text.chars() {
        if character.is_ascii_alphabetic() {
            has_letter = true;
            continue;
        }

        if character.is_ascii_whitespace() {
            continue;
        }

        // Allow a small, conservative set of punctuation characters
        if matches!(
            character,
            '.' | ',' | '!' | '?' | ';' | ':' | '\'' | '"' | '-' | '(' | ')'
        ) {
            continue;
        }

        // Reject digits, emojis, non-ASCII scripts, and other symbols
        return false;
    }

    has_letter
}
