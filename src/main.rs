use clap::{Arg, Command};
use rusqlite::Connection;
use std::time::Instant;
use prettytable::{Table, row};
use rust_russian_wordle::{Wordle, WordleQuery, load_words_from_query, is_valid_pattern, parse_pattern};

fn main() -> rusqlite::Result<()> {
    let start = Instant::now();

    let matches = Command::new("ruwordle")
        .version("1.0")
        .author("Alan Duncan <duncan.alan@me.com>")
        .about("This tool is meant to offer suggested words for the Russian version of Wordle.")
        .arg(
            Arg::new("pattern")
                .short('p')
                .long("pattern")
                .value_name("PATTERN")
                .help("Letter patterns for known/unknown")
                .required(true)
                .action(clap::ArgAction::Append)
                .num_args(1..),
        )
        .arg(
            Arg::new("rejects")
                .short('r')
                .long("rejects")
                .value_name("REJECTS")
                .help("Comma-delimited rejected letters")
                .num_args(0..=1),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("limit")
                .value_name("LIMIT")
                .help("Limit the number of suggestions")
                .num_args(1),
        )
        .get_matches();

    let patterns: Vec<String> = matches.get_many::<String>("pattern").unwrap().cloned().collect();
    let mut pattern_lengths_valid = true;

    for pattern in &patterns {
        if !is_valid_pattern(pattern) {
            eprintln!("Error: Incorrect pattern format");
            pattern_lengths_valid = false;
        }
    }

    if !pattern_lengths_valid {
        std::process::exit(1);
    }

    let mut all_rejects: Vec<char> = vec![];
    let mut validated_patterns = vec![];
    for pattern in &patterns {
        let (modified_pattern, extracted_rejects) = parse_pattern(pattern);
        all_rejects.extend(extracted_rejects); // Collect all extracted rejects
        validated_patterns.push(modified_pattern);
    }
    let rejects = matches.get_one::<String>("rejects").map(String::as_str).unwrap_or("");
    // Add additional rejects provided via --rejects
    all_rejects.extend(rejects.chars());

    let rejects_string: String = all_rejects.iter().collect();

    let limit = matches.get_one::<String>("limit").unwrap_or(&"0".to_string()).parse::<usize>().unwrap_or(0);

    let conn = Connection::open("/Users/alan/Documents/dev/databases/stalin.db")?;

    let mut results = None;

    for pattern in validated_patterns {
        match WordleQuery::new(&pattern, &rejects_string) {
            Ok(wordle_query) => {
                let query = wordle_query.build_query();
                let words = load_words_from_query(&query, &conn)?;

                results = match results {
                    None => Some(words),
                    Some(existing_results) => Some(existing_results.intersection(&words).cloned().collect()),
                };

                if results.as_ref().unwrap().is_empty() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut wordles: Vec<Wordle> = if let Some(words) = results {
        words.into_iter().map(Wordle::new).collect()
    } else {
        Vec::new()
    };

    wordles.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    if limit > 0 && wordles.len() > limit {
        wordles.truncate(limit);
    }

    let mut table = Table::new();
    table.add_row(row!["lemma", "score"]);
    for wordle in &wordles {
        table.add_row(row![wordle.lemma, wordle.score as u64]);
    }

    table.printstd();

    let duration = start.elapsed();
    println!("Elapsed time: {:.3}s", duration.as_secs_f64());

    Ok(())
}
