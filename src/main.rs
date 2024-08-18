use clap::{Arg, Command};
use rusqlite::Connection;
use std::time::Instant;
use prettytable::{Table, row};
use std::fs;
use std::path::PathBuf;
use dirs;
use serde_json::Value;
use rust_russian_wordle::{is_valid_pattern, parse_pattern, WordleQuery, load_words_from_query, Wordle};

fn load_config() -> Option<String> {
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/rust_russian_wordle/config.json");

    if let Ok(config_content) = fs::read_to_string(config_path) {
        if let Ok(json) = serde_json::from_str::<Value>(&config_content) {
            if let Some(db_path) = json.get("db_path") {
                return db_path.as_str().map(|s| s.to_string());
            }
        }
    }
    None
}

fn save_config(db_path: &str) {
    let config_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/rust_russian_wordle");

    let config_file = config_dir.join("config.json");

    // Create the directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("Failed to create config directory: {}", e);
        return;
    }

    // Save the database path to the config file
    let config_data = serde_json::json!({ "db_path": db_path });
    if let Err(e) = fs::write(config_file, config_data.to_string()) {
        eprintln!("Failed to write config file: {}", e);
    }
}

fn remove_config() {
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/rust_russian_wordle/config.json");

    // Attempt to remove the configuration file
    if fs::remove_file(config_path).is_err() {
        eprintln!("Failed to remove the configuration file or file does not exist.");
    } else {
        println!("Database path has been removed.");
    }
}

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
                .required(false)
                .action(clap::ArgAction::Append)
                .num_args(1..),
        )
        .arg(
            Arg::new("rejects")
                .short('r')
                .long("rejects")
                .value_name("REJECTS")
                .help("Comma-separated list of rejected letters"),
        )
        .arg(
            Arg::new("limit")
                .long("limit")
                .value_name("LIMIT")
                .help("Limit the number of words displayed")
                .required(false)
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("dbpath")
                .long("dbpath")
                .value_name("DB_PATH")
                .help("Path to the database")
                .required(false),
        )
        .arg(
            Arg::new("setdbpath")
                .long("setdbpath")
                .value_name("SET_DB_PATH")
                .help("Set and store the path to the database")
                .required(false),
        )
        .arg(
            Arg::new("remove_dbpath")
                .long("remove_dbpath")
                .help("Remove the stored database path")
                .required(false)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Check if we are setting or removing the dbpath
    if matches.contains_id("setdbpath") {
        if let Some(new_db_path) = matches.get_one::<String>("setdbpath") {
            save_config(new_db_path);
            println!("Database path set to: {}", new_db_path);
        }
        return Ok(());
    }

    if matches.get_flag("remove_dbpath") {
        remove_config();
        return Ok(());
    }

    // Load the DB path from the command line or config file
    let db_path = if let Some(db_path) = matches.get_one::<String>("dbpath") {
        db_path.to_string()
    } else if let Some(stored_db_path) = load_config() {
        stored_db_path
    } else {
        eprintln!("Error: No database path set. Use --setdbpath to set the database path.");
        return Ok(());
    };

    // Ensure pattern is provided when neither setdbpath nor remove_dbpath is provided
    if !matches.contains_id("pattern") {
        eprintln!("Error: --pattern is required unless setting or removing the database path.");
        return Ok(());
    }

    let patterns: Vec<String> = matches
        .get_many::<String>("pattern")
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    println!("Using database path: {}", db_path);
    println!("Using patterns: {:?}", patterns);

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

    let limit = *matches.get_one::<usize>("limit").unwrap_or(&10); 
    let conn = Connection::open(db_path)?;

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
