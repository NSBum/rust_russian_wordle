// External Dependencies
use rusqlite::{Connection, Result};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

// Error Definitions
#[derive(Error, Debug)]
pub enum WordleQueryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Regex pattern not valid")]
    InvalidRegexPattern(String),
}

// Struct for Wordle Word and Methods
pub struct Wordle {
    pub lemma: String,
    pub score: f64,
}

impl Wordle {
    /// Replace Cyrillic 'ё' with 'е'
    pub fn replace_yo(lemma: &str) -> String {
        lemma.replace('ё', "е")
    }

    /// Calculate the score of a word based on letter frequencies
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

    /// Create a new Wordle instance
    pub fn new(lemma: String) -> Self {
        let lemma = Self::replace_yo(&lemma);
        let ru_letter_freqs = Self::init_letter_freqs();
        let score = Self::calculate_score(&lemma, &ru_letter_freqs);
        Wordle { lemma, score }
    }

    /// Initialize letter frequencies for Russian letters
    pub fn init_letter_freqs() -> HashMap<char, f64> {
        vec![
            ('о', 10.97), ('е', 8.45), ('а', 8.01), ('и', 7.35), ('н', 6.70), ('т', 6.26),
            ('с', 5.47), ('л', 4.97), ('в', 4.53), ('р', 4.40), ('к', 3.49), ('м', 3.21),
            ('д', 2.98), ('п', 2.81), ('ы', 2.10), ('у', 2.08), ('б', 1.92), ('я', 1.79),
            ('ь', 1.74), ('г', 1.70), ('з', 1.65), ('ч', 1.44), ('й', 1.21), ('ж', 1.01),
            ('х', 0.95), ('ш', 0.72), ('ю', 0.49), ('ц', 0.48), ('э', 0.32), ('щ', 0.31),
            ('ф', 0.26), ('ъ', 0.04),
        ].into_iter().collect()
    }
}

// Struct for WordleQuery and Methods
#[derive(Debug)]
pub struct WordleQuery {
    pub pattern: String,
    pub rejects: Vec<char>,
}

impl WordleQuery {
    /// Create a new WordleQuery instance, validate input pattern and rejects
    pub fn new(pattern: &str, rejects: &str) -> Result<Self, WordleQueryError> {
        println!("Pattern = {}", pattern);

        if !is_valid_pattern(pattern) {
            return Err(WordleQueryError::QueryError(
                "Pattern must contain exactly 5 Cyrillic (or *) characters.".to_string(),
            ));
        }

        let rejects = process_rejects(rejects);
        Ok(WordleQuery { pattern: pattern.to_string(), rejects })
    }

    /// Extracts rejects from the pattern and modifies the pattern
    pub fn extract_rejects(pattern: &mut String) -> Result<Vec<char>, WordleQueryError> {
        let re = Regex::new(r"_([\p{Cyrillic}])").unwrap();
        let mut collected_letters = Vec::new();
        *pattern = re.replace_all(pattern, |captures: &regex::Captures| {
            if let Some(c) = captures.get(1) {
                collected_letters.push(c.as_str().chars().next().unwrap());
            }
            ""
        }).to_string();
        Ok(collected_letters)
    }

    /// Build SQL query for the Wordle database
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

// Utility Functions
pub fn append_chars_to_comma_delimited_str(rejects: &str, chars_to_add: Vec<char>) -> String {
    let mut result = String::from(rejects);
    if !result.is_empty() {
        result.push(',');
    }
    for (i, c) in chars_to_add.iter().enumerate() {
        if i > 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

pub fn parse_pattern(input: &str) -> (String, Vec<char>) {
    let re = Regex::new(r"_([\p{Cyrillic}])").unwrap();
    let mut collected_rejects = Vec::new();
    let modified_pattern = re.replace_all(input, |captures: &regex::Captures| {
        if let Some(c) = captures.get(1) {
            collected_rejects.push(c.as_str().chars().next().unwrap());
        }
        "*"
    }).to_string();
    (modified_pattern, collected_rejects)
}

pub fn load_words_from_query(query: &str, conn: &Connection) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare(query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;
    let words: HashSet<String> = word_iter.flatten().collect();
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
        .flat_map(|c| c.to_lowercase())
        .map(convert_ye_to_yo)
        .map(convert_latin_to_cyrillic)
        .collect()
}

pub fn is_valid_pattern(pattern: &str) -> bool {
    let re = Regex::new(r"_([a-яё])").unwrap();
    let modified_pattern = re.replace_all(pattern, |_: &regex::Captures| {
        "*"
    });
    let pattern_length = UnicodeSegmentation::graphemes(&*modified_pattern, true).count();
    pattern_length == 5
}

// Unit Tests
#[test]
fn test_parse_pattern_with_multiple_cyrillic_rejects() {
    let (_, rejects) = parse_pattern("_о*_т**А");
    assert_eq!(rejects, vec!['о', 'т']);
}

#[test]
fn test_parse_pattern_modifies_pattern_correctly_with_cyrillic_rejects() {
    let (modified_pattern, _) = parse_pattern("_о*_т*А");
    assert_eq!(modified_pattern, "****А");
}

#[test]
fn test_parse_pattern_with_only_wildcards() {
    let (_, rejects) = parse_pattern("*****");
    assert_eq!(rejects.len(), 0);
}

#[test]
fn test_parse_pattern_with_only_cyrillic_letters() {
    let (modified_pattern, _) = parse_pattern("АБВГД");
    assert_eq!(modified_pattern, "АБВГД");
}

#[test]
fn test_parse_pattern_with_no_rejects_and_only_wildcards_and_letters() {
    let (modified_pattern, _) = parse_pattern("А*Б*В");
    assert_eq!(modified_pattern, "А*Б*В");
}

#[test]
fn test_parse_pattern_with_cyrillic_reject_at_start() {
    let (_, rejects) = parse_pattern("_о****");
    assert_eq!(rejects, vec!['о']);
}

#[test]
fn test_parse_pattern_with_cyrillic_reject_at_end() {
    let (_, rejects) = parse_pattern("****_т");
    assert_eq!(rejects, vec!['т']);
}

#[test]
fn test_append_chars_to_str() {
    let rejects = "о,с,и,н";
    let additional_rejects = vec!['а','т'];
    let actual = append_chars_to_comma_delimited_str(rejects, additional_rejects);
    let expected = "о,с,и,н,а,т";
    assert_eq!(actual, expected);
}

#[test]
fn test_extract_rejects() {
    let mut input = String::from("**_н**");
    let collected_letters = WordleQuery::extract_rejects(&mut input).unwrap();
    assert_eq!(input, "****");
    assert_eq!(collected_letters, vec!['н']);
}

#[test]
fn test_valid_pattern_length_with_rejects() {
    let pattern = "_о_т***";
    assert_eq!(is_valid_pattern(pattern), true);
}

#[test]
fn test_not_valid_pattern_length_with_rejects() {
    let pattern = "_а_б_ф_рдт";
    assert_eq!(is_valid_pattern(pattern), false);
}

#[test]
fn test_valid_pattern_length_without_rejects() {
    let pattern = "*****";
    assert_eq!(is_valid_pattern(pattern), true);
}

#[test]
fn test_valid_pattern_without_rejcts_has_letters() {
    let pattern = "**И*а";
    assert_eq!(is_valid_pattern(pattern), true);
}
