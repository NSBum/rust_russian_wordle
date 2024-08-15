# rust_russian_wordle

`rust_russian_wordle` is a command-line tool designed to assist with solving the Russian version of the popular game Wordle. It allows users to input known letter patterns and rejected letters to generate possible valid words in Russian. The tool leverages a SQLite database containing Russian words and calculates the probability scores for each suggestion based on the frequency of Cyrillic characters.

## Features

- Accepts letter patterns with wildcards (`*`) for unknown positions.
- Supports extraction and processing of rejected Cyrillic letters.
- Scores potential solutions based on the frequency of letters in the Russian alphabet.
- Provides SQL query generation for efficient word lookups from a SQLite database.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Examples](#examples)
- [How It Works](#how-it-works)
- [Development](#development)

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/) (for building and running the project)
- [SQLite](https://www.sqlite.org/) (to interact with the word database)

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/rust_russian_wordle.git
   cd rust_russian_wordle

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Ensure you have a SQLite database with Russian words in it. Place the database in the expected location (default: `/path/to/database.db`).

## Usage

To run the tool, use the following command:

```bash
./target/release/rust_russian_wordle --pattern "_о*_т**А" --rejects "е,и" --limit 10
```

### Command-Line Arguments

- `--pattern (-p)`: A string representing known letters and positions, with `*` as a wildcard for unknown letters. For example, `*о*т*`.
- `--rejects (-r)`: A comma-separated list of Cyrillic letters that are not present in the word.
- `--limit (-l)`: Limits the number of word suggestions returned.

### Options

| Option   | Description                                                                                       | Example                       |
|----------|---------------------------------------------------------------------------------------------------|-------------------------------|
| `-p`     | Pattern of the word with known and unknown positions (`*` for unknown positions).                  | `"_о*_т*А"`                   |
| `-r`     | Rejected letters in the word (comma-delimited).                                                    | `"е,и,ж"`                     |
| `-l`     | Limit the number of word suggestions returned.                                                     | `10`                          |

## Examples

### Example 1: Simple Query with Known Letters and Rejects

```bash
./rust_russian_wordle -p "_о*_т*А" -r "е,и,ж" -l 10
```

- **Pattern**: `"_о*_т*А"` (The word starts with an unknown letter, then `о`, followed by any letter, `т`, another wildcard, and ending with `А`.)
- **Rejects**: `"е,и,ж"` (The letters `е`, `и`, and `ж` should not be present in the word.)

### Example 2: Fully Wildcard Pattern

```bash
./rust_russian_wordle -p "*****" -r "о,с,м,п" -l 5
```

- **Pattern**: `"*****"` (All letters are unknown.)
- **Rejects**: `"о,с,м,п"` (Rejects the letters `о`, `с`, `м`, and `п`.)
- **Limit**: `5` (Limits the output to 5 suggestions.)

## How It Works

The tool operates by using patterns and reject letters to generate SQL queries that search for words in the SQLite database. Each word is then scored based on the frequency of its letters in the Russian language, and the results are sorted and returned.

### Core Functions

- **Pattern Parsing**: Patterns are processed to replace `_<Cyrillic letter>` sequences with wildcards (`*`), and Cyrillic letters prefixed by `_` are collected as rejects.
- **Reject Processing**: The reject letters are filtered and converted to ensure they match the expected Cyrillic characters.
- **Word Scoring**: Words are scored based on letter frequencies, with rarer letters giving lower scores and more common letters giving higher scores.
- **SQL Query Generation**: A query is dynamically constructed based on the pattern and rejects to retrieve words from the database that match the criteria.

### Behind the Scenes

- **SQLite Database**: The words are stored in a SQLite database, and the program queries this database using SQL generated from the input pattern and rejects.
- **Letter Frequency Calculation**: The frequency of each Russian letter is used to calculate the likelihood of a word being the correct answer based on the known statistics.

## Development

### Running Tests

To run tests for the project, use:

```bash
cargo test
```

Tests are implemented for core functionalities such as pattern parsing, reject extraction, and SQL query generation.

### Contribution

Feel free to open issues or submit pull requests if you'd like to contribute to improving the tool.

---

Happy solving!
