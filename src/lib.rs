use rusqlite::{Connection, Result};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WordleQueryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Query error: {0}")]
    QueryError(String),
}

pub struct Wordle {
    pub lemma: String,
    pub score: f64,
}

impl Wordle {
    pub fn replace_yo(lemma: &str) -> String {
        lemma.replace('ё', "е")
    }

    pub fn calculate_score(lemma: &str, ru_letter_freqs: &HashMap<char, f64>) -> f64 {
        let mut score = 1.0;

        for c in lemma.chars() {
            if let Some(&letter_score) = ru_letter_freqs.get(&c) {
                score *= letter_score;
            } else {
                break;
            }
        }

        score
    }

    pub fn new(lemma: String) -> Self {
        let lemma = Self::replace_yo(&lemma);
        let ru_letter_freqs = Self::init_letter_freqs();
        let score = Self::calculate_score(&lemma, &ru_letter_freqs);

        Wordle {
            lemma,
            score,
        }
    }

    pub fn init_letter_freqs() -> HashMap<char, f64> {
        let frequencies = vec![
            ('о', 10.97),
            ('е', 8.45),
            ('а', 8.01),
            ('и', 7.35),
            ('н', 6.70),
            ('т', 6.26),
            ('с', 5.47),
            ('л', 4.97),
            ('в', 4.53),
            ('р', 4.40),
            ('к', 3.49),
            ('м', 3.21),
            ('д', 2.98),
            ('п', 2.81),
            ('ы', 2.10),
            ('у', 2.08),
            ('б', 1.92),
            ('я', 1.79),
            ('ь', 1.74),
            ('г', 1.70),
            ('з', 1.65),
            ('ч', 1.44),
            ('й', 1.21),
            ('ж', 1.01),
            ('х', 0.95),
            ('ш', 0.72),
            ('ю', 0.49),
            ('ц', 0.48),
            ('э', 0.32),
            ('щ', 0.31),
            ('ф', 0.26),
            ('ъ', 0.04),
        ];

        frequencies.into_iter().collect()
    }
}

#[derive(Debug)]
pub struct WordleQuery {
    pub pattern: String,
    pub rejects: Vec<char>,
}

impl WordleQuery {
    pub fn new(pattern: &str, rejects: &str) -> Result<Self, WordleQueryError> {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let valid_pattern = pattern_chars.len() == 5 && pattern_chars.iter().all(|&c| {
            c == '*' || c.is_cyrillic_char()
        });

        if !valid_pattern {
            return Err(WordleQueryError::QueryError(String::from(
                "Pattern must contain exactly 5 Cyrillic (or *) characters.",
            )));
        }

        let rejects = process_rejects(rejects);

        Ok(WordleQuery {
            pattern: pattern.to_string(),
            rejects,
        })
    }

    pub fn build_query(&self) -> String {
        let mut query = String::from("SELECT w.word FROM words w WHERE LENGTH(w.word) = 5");
        query.push_str(" AND w.word GLOB '[а-я]*'");
        query.push_str(" AND w.word NOT LIKE '%-%'");
        query.push_str(" AND w.word NOT LIKE '%.%'");

        for (i, c) in self.pattern.chars().enumerate() {
            match c {
                '*' => {}
                _ if c.is_uppercase() => {
                    query.push_str(&format!(" AND SUBSTR(w.word, {}, 1) = '{}'", i + 1, c.to_lowercase()));
                }
                _ if c.is_lowercase() => {
                    query.push_str(&format!(" AND w.word LIKE '%{}%' AND SUBSTR(w.word, {}, 1) != '{}'", c, i + 1, c));
                }
                _ => {}
            }
        }

        for reject in &self.rejects {
            query.push_str(&format!(" AND w.word NOT LIKE '%{}%'", reject));
        }

        query
    }
}

pub fn load_words_from_query(query: &str, conn: &Connection) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare(query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;

    let mut words = HashSet::new();
    for word_result in word_iter {
        if let Ok(word) = word_result {
            words.insert(word);
        }
    }

    Ok(words)
}

pub fn convert_latin_to_cyrillic(c: char) -> char {
    match c {
        'e' => 'е', // Latin 'e' to Cyrillic 'е'
        'o' => 'о', // Latin 'o' to Cyrillic 'о'
        _ => c,     // Leave other characters unchanged
    }
}

pub fn convert_ye_to_yo(c: char) -> char {
    match c {
        'ё' => 'е',
        _ => c,
    }
}

pub fn process_rejects(rejects: &str) -> Vec<char> {
    rejects.chars()
        .filter(|&c| c != ',')
        .flat_map(|c| c.to_lowercase())// Step 1: Filter out commas
        .map(convert_ye_to_yo)
        .map(convert_latin_to_cyrillic)   // Step 2: Convert remaining characters
           // Handle multi-character expansion

        .collect()                        // Step 3: Collect the results
}

trait CharExt {
    fn is_cyrillic_char(self) -> bool;
}

impl CharExt for char {
    fn is_cyrillic_char(self) -> bool {
        ('\u{0400}'..='\u{04FF}').contains(&self) || self == 'ё'
    }
}
