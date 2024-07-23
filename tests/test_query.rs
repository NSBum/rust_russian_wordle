use rusqlite::{Connection, Result};
use rust_russian_wordle::{WordleQuery, process_rejects, WordleQueryError};

type TestResult = Result<(), WordleQueryError>;

#[test]
fn test_query_excludes_uppercase_words() -> TestResult {
    // Create an in-memory SQLite database
    let conn = Connection::open_in_memory()?;

    // Create a table and insert test data
    conn.execute(
        "CREATE TABLE words (word TEXT NOT NULL)",
        [],
    )?;
    let test_data = vec!["мирно", "Мирно", "слово", "Слово"];
    for word in test_data {
        conn.execute(
            "INSERT INTO words (word) VALUES (?1)",
            [&word],
        )?;
    }

    // Build the query using WordleQuery
    let wordle_query = WordleQuery::new("*****", "")?;
    let query = wordle_query.build_query();
    //println!("Generated Query: {}", query);

    // Execute the query
    let mut stmt = conn.prepare(&query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;

    // Collect the results
    let results: Vec<String> = word_iter.filter_map(|word_result| word_result.ok()).collect();
    //println!("Query Results: {:?}", results);

    // Assert that the results do not contain words starting with an uppercase letter
    let expected_results: Vec<&str> = vec!["мирно", "слово"];
    assert_eq!(results, expected_results);

    Ok(())
}

#[test]
fn test_query_returns_only_5_letter_words() -> TestResult {
    // Create an in-memory SQLite database
    let conn = Connection::open_in_memory()?;

    // Create a table and insert test data
    conn.execute(
        "CREATE TABLE words (word TEXT NOT NULL)",
        [],
    )?;
    let test_data = vec!["мирно", "Привет", "слово", "Слово", "тест", "тестирование"];
    for word in test_data {
        conn.execute(
            "INSERT INTO words (word) VALUES (?1)",
            [&word],
        )?;
    }

    // Build the query using WordleQuery
    let wordle_query = WordleQuery::new("*****", "")?;
    let query = wordle_query.build_query();
    //println!("Generated Query: {}", query);

    // Execute the query
    let mut stmt = conn.prepare(&query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;

    // Collect the results
    let results: Vec<String> = word_iter.filter_map(|word_result| word_result.ok()).collect();
    //println!("Query Results: {:?}", results);

    // Assert that all words in the results have a length of 5
    for word in &results {
        assert_eq!(word.chars().count(), 5, "Word '{}' does not have 5 letters", word);
    }

    Ok(())
}

#[test]
fn test_query_excludes_words_with_rejected_letters() -> TestResult {
    // Create an in-memory SQLite database
    let conn = Connection::open_in_memory()?;

    // Create a table and insert test data
    conn.execute(
        "CREATE TABLE words (word TEXT NOT NULL)",
        [],
    )?;
    let test_data = vec!["мирно", "слово", "тесто", "гром", "кубик"];
    for word in test_data {
        conn.execute(
            "INSERT INTO words (word) VALUES (?1)",
            [&word],
        )?;
    }

    // Build the query using WordleQuery with rejected letters
    let wordle_query = WordleQuery::new("*****", "о,е")?;
    let query = wordle_query.build_query();
    //println!("Generated Query: {}", query);

    // Execute the query
    let mut stmt = conn.prepare(&query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;

    // Collect the results
    let results: Vec<String> = word_iter.filter_map(|word_result| word_result.ok()).collect();
    //println!("Query Results: {:?}", results);

    // Assert that the results do not contain words with rejected letters 'о' or 'е'
    let expected_results: Vec<&str> = vec!["кубик"];
    assert_eq!(results, expected_results);

    Ok(())
}

#[test]
fn test_query_excludes_words_with_yellow_letters_in_correct_position() -> TestResult {
    // Create an in-memory SQLite database
    let conn = Connection::open_in_memory()?;

    // Create a table and insert test data
    conn.execute(
        "CREATE TABLE words (word TEXT NOT NULL)",
        [],
    )?;
    let test_data = vec!["мирно", "минор", "слово", "морни", "ранки"];
    for word in test_data {
        conn.execute(
            "INSERT INTO words (word) VALUES (?1)",
            [&word],
        )?;
    }

    // Build the query using WordleQuery with a yellow letter 'н' not in the 3rd position
    let wordle_query = WordleQuery::new("**н**", "")?;
    let query = wordle_query.build_query();
    //println!("Generated Query: {}", query);

    // Execute the query
    let mut stmt = conn.prepare(&query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;

    // Collect the results
    let results: Vec<String> = word_iter.filter_map(|word_result| word_result.ok()).collect();
    //println!("Query Results: {:?}", results);

    // Assert that the results do not contain words with 'н' in the correct 3rd position
    let expected_results: Vec<&str> = vec!["мирно", "морни"];
    assert_eq!(results, expected_results);

    Ok(())
}

#[test]
fn test_query_with_limit() -> TestResult {
    // Create an in-memory SQLite database
    let conn = Connection::open_in_memory()?;

    // Create a table and insert test data
    conn.execute(
        "CREATE TABLE words (word TEXT NOT NULL)",
        [],
    )?;

    // Insert more than 10 words for the test
    let test_data = vec![
        "мирно", "слово", "пятью", "игрок", "шахи",
        "домик", "улица", "горы", "книга", "рыбак",
        "цветы", "почта", "карта", "снегу", "птицы"
    ];

    for word in test_data {
        conn.execute(
            "INSERT INTO words (word) VALUES (?1)",
            [&word],
        )?;
    }

    // Build the query using WordleQuery with a limit of 10
    let wordle_query = WordleQuery::new("*****", "")?;
    let query = format!("{} LIMIT 10", wordle_query.build_query());
    //println!("Generated Query: {}", query);

    // Execute the query
    let mut stmt = conn.prepare(&query)?;
    let word_iter = stmt.query_map([], |row| {
        let word: String = row.get(0)?;
        Ok(word)
    })?;

    // Collect the results
    let results: Vec<String> = word_iter.filter_map(|word_result| word_result.ok()).collect();
    //println!("Query Results: {:?}", results);

    // Assert that the results contain only 10 items
    assert_eq!(results.len(), 10, "Expected 10 items, but got {}", results.len());

    Ok(())
}

#[test]
fn test_process_rejects() {
    // mixed cases and errors in Latin vs Cyrillic
    let input = "ё,E,д,Я,O";
    // should be all Cyrillic
    let expected: Vec<char> = vec!['е','е','д','я','о'];
    assert_eq!(process_rejects(input), expected);
}

#[test]
fn test_wordle_query_invalid_pattern_too_short() -> Result<(), WordleQueryError> {
    let pattern = "****"; // 4 characters
    let rejects = "EoABCё";
    let result = WordleQuery::new(pattern, rejects);
    assert!(result.is_err());
    match result.unwrap_err() {
        WordleQueryError::QueryError(msg) => assert_eq!(msg, "Pattern must contain exactly 5 Cyrillic (or *) characters."),
        _ => panic!("Unexpected error type"),
    }
    Ok(())
}
#[test]
fn test_wordle_query_invalid_pattern_way_too_long() -> Result<(), WordleQueryError> {
    let pattern = "яшертыуидйд";
    let rejects = "к";
    let result = WordleQuery::new(pattern,rejects);
    assert!(result.is_err());
    match result.unwrap_err() {
        WordleQueryError::QueryError(msg) => assert_eq!(msg, "Pattern must contain exactly 5 Cyrillic (or *) characters."),
        _ => panic!("Unexpected error type"),
    }
    Ok(())
}

#[test]
fn test_wordle_query_invalid_pattern_just_too_long() -> Result<(), WordleQueryError> {
    let pattern = "**яшей";
    let rejects = "к";
    let result = WordleQuery::new(pattern,rejects);
    assert!(result.is_err());
    match result.unwrap_err() {
        WordleQueryError::QueryError(msg) => assert_eq!(msg, "Pattern must contain exactly 5 Cyrillic (or *) characters."),
        _ => panic!("Unexpected error type"),
    }
    Ok(())
}

