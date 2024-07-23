use clap::{Arg, Command};
use rusqlite::Connection;
use std::time::Instant;
use prettytable::{Table, row};
use unicode_segmentation::UnicodeSegmentation;
use rust_russian_wordle::{Wordle, WordleQuery, load_words_from_query};

fn main() -> rusqlite::Result<()> {
    let start = Instant::now();

    let matches = Command::new("ruwordle")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
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
        let pattern_length = UnicodeSegmentation::graphemes(pattern.as_str(), true).count();
        if pattern_length != 5 {
            eprintln!("Error: Each pattern must be exactly 5 characters long. Provided pattern length: {} - {}", pattern_length, pattern);
            pattern_lengths_valid = false;
        }
    }

    if !pattern_lengths_valid {
        std::process::exit(1);
    }

    let rejects = matches.get_one::<String>("rejects").map(String::as_str).unwrap_or("");
    let limit = matches.get_one::<String>("limit").unwrap_or(&"0".to_string()).parse::<usize>().unwrap_or(0);

    let conn = Connection::open("/Users/alan/Documents/dev/databases/stalin.db")?;

    let mut results = None;

    for pattern in patterns {
        let wordle_query = WordleQuery::new(&pattern, rejects);
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
