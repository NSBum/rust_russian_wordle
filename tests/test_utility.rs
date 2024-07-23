use rust_russian_wordle::{Wordle};
use rust_russian_wordle::convert_latin_to_cyrillic;

#[test]
fn test_replace_yo() {
    let input = "ёлка";
    let expected = "елка";
    assert_eq!(Wordle::replace_yo(input), expected);
}

#[test]
fn test_calculate_score() {
    let lemma = "привет";
    let mut ru_letter_freqs = std::collections::HashMap::new();
    ru_letter_freqs.insert('п', 1.0);
    ru_letter_freqs.insert('р', 1.0);
    ru_letter_freqs.insert('и', 1.0);
    ru_letter_freqs.insert('в', 1.0);
    ru_letter_freqs.insert('е', 1.0);
    ru_letter_freqs.insert('т', 1.0);
    let score = Wordle::calculate_score(lemma, &ru_letter_freqs);
    assert_eq!(score, 1.0);
}

#[test]
fn test_convert_latin_to_cyrillic() {
    // Test conversion of Latin 'e' to Cyrillic 'е'
    assert_eq!(convert_latin_to_cyrillic('e'), 'е');
    // Test conversion of Latin 'o' to Cyrillic 'о'
    assert_eq!(convert_latin_to_cyrillic('o'), 'о');
    // Test that other characters remain unchanged
    assert_eq!(convert_latin_to_cyrillic('a'), 'a');
    assert_eq!(convert_latin_to_cyrillic('z'), 'z');
    assert_eq!(convert_latin_to_cyrillic('я'), 'я'); // Cyrillic character
}
