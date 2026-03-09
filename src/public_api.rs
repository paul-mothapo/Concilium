use std::collections::BTreeSet;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::corpus::CorpusLoadReport;

const DATAMUSE_API: &str = "https://api.datamuse.com/words";
const QUOTABLE_API: &str = "https://api.quotable.io/quotes/random";
const POETRY_DB_API: &str = "https://poetrydb.org/random";
const BACON_IPSUM_API: &str = "https://baconipsum.com/api/";

pub fn fetch_public_api_corpus(context_window: usize) -> Result<CorpusLoadReport, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("failed to build HTTP client: {error}"))?;

    let word_target = context_window;
    let sentence_target = (context_window / 25).clamp(10, 400);

    let words_result = fetch_datamuse_words(&client, word_target);
    let passages_result = fetch_public_passages(&client, sentence_target);

    let mut api_sources = Vec::new();
    let mut errors = Vec::new();

    let words = match words_result {
        Ok(words) if !words.is_empty() => {
            api_sources.push(format!(
                "Datamuse batched prefixes toward {word_target} words"
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

fn fetch_datamuse_words(client: &Client, target: usize) -> Result<Vec<String>, String> {
    let batch_limit = target
        .saturating_add(25)
        .div_ceil(26)
        .saturating_add(50)
        .clamp(250, 1000);
    let secondary_batch_limit = (target / 100).clamp(50, 200);
    let mut words = BTreeSet::new();
    let mut errors = Vec::new();

    for prefix in ('a'..='z').map(|letter| format!("{letter}*")) {
        match fetch_datamuse_words_for_pattern(client, &prefix, batch_limit) {
            Ok(batch) => {
                words.extend(batch);
                if words.len() >= target {
                    break;
                }
            }
            Err(error) => errors.push(error),
        }
    }

    if words.len() < target {
        'outer: for first in 'a'..='z' {
            for second in 'a'..='z' {
                let prefix = format!("{first}{second}*");
                match fetch_datamuse_words_for_pattern(client, &prefix, secondary_batch_limit) {
                    Ok(batch) => {
                        words.extend(batch);
                        if words.len() >= target {
                            break 'outer;
                        }
                    }
                    Err(error) => errors.push(error),
                }
            }
        }
    }

    if words.is_empty() {
        return Err(errors.join(" | "));
    }

    Ok(words.into_iter().take(target).collect())
}

fn fetch_datamuse_words_for_pattern(
    client: &Client,
    pattern: &str,
    limit: usize,
) -> Result<Vec<String>, String> {
    let response = client
        .get(DATAMUSE_API)
        .query(&[("sp", pattern), ("max", &limit.to_string())])
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Datamuse request failed for pattern {pattern}: {error}"))?;

    let payload = response.json::<Vec<DatamuseWord>>().map_err(|error| {
        format!("Datamuse response parse failed for pattern {pattern}: {error}")
    })?;

    Ok(payload
        .into_iter()
        .map(|item| item.word)
        .filter(|word| {
            word.chars()
                .all(|character| character.is_ascii_alphabetic())
        })
        .collect())
}

fn fetch_quotable_passages(client: &Client, limit: usize) -> Result<Vec<String>, String> {
    let response = client
        .get(QUOTABLE_API)
        .query(&[
            ("limit", limit.to_string()),
            ("minLength", "120".to_owned()),
            ("maxLength", "280".to_owned()),
        ])
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Quotable request failed: {error}"))?;

    let payload = response
        .json::<Vec<QuotableQuote>>()
        .map_err(|error| format!("Quotable response parse failed: {error}"))?;

    Ok(payload.into_iter().map(|quote| quote.content).collect())
}

fn fetch_public_passages(client: &Client, target: usize) -> Result<(Vec<String>, String), String> {
    let mut errors = Vec::new();

    match fetch_quotable_passages(client, target.min(50)) {
        Ok(sentences) if !sentences.is_empty() => {
            return Ok((
                sentences,
                format!(
                    "Quotable /quotes/random?limit={}&minLength=120&maxLength=280",
                    target.min(50)
                ),
            ));
        }
        Ok(_) => errors.push("Quotable returned no passages".to_owned()),
        Err(error) => errors.push(error),
    }

    match fetch_poetrydb_passages(client, target) {
        Ok(sentences) if !sentences.is_empty() => {
            return Ok((
                sentences,
                format!("PoetryDB batched random toward {target} passages"),
            ));
        }
        Ok(_) => errors.push("PoetryDB returned no passages".to_owned()),
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

fn fetch_poetrydb_passages(client: &Client, target: usize) -> Result<Vec<String>, String> {
    let mut sentences = Vec::new();
    let mut errors = Vec::new();
    let mut remaining = target;

    while remaining > 0 {
        let batch_size = remaining.min(100);
        let response = match client
            .get(format!("{POETRY_DB_API}/{batch_size}"))
            .send()
            .and_then(|response| response.error_for_status())
        {
            Ok(response) => response,
            Err(error) => {
                errors.push(format!("PoetryDB request failed: {error}"));
                break;
            }
        };

        let payload = match response.json::<Vec<PoetryDbPoem>>() {
            Ok(payload) => payload,
            Err(error) => {
                errors.push(format!("PoetryDB response parse failed: {error}"));
                break;
            }
        };

        let mut batch = payload
            .into_iter()
            .map(|poem| poem.lines.join(" "))
            .filter(|poem| !poem.trim().is_empty())
            .collect::<Vec<_>>();

        if batch.is_empty() {
            break;
        }

        remaining = remaining.saturating_sub(batch.len());
        sentences.append(&mut batch);
    }

    if sentences.is_empty() {
        return Err(errors.join(" | "));
    }

    Ok(sentences.into_iter().take(target).collect())
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
        .map(|paragraphs| paragraphs.into_iter().take(target).collect())
        .map_err(|error| format!("Bacon Ipsum response parse failed: {error}"))
}

#[derive(Clone, Debug, Deserialize)]
struct DatamuseWord {
    word: String,
}

#[derive(Clone, Debug, Deserialize)]
struct QuotableQuote {
    content: String,
}

#[derive(Clone, Debug, Deserialize)]
struct PoetryDbPoem {
    lines: Vec<String>,
}
