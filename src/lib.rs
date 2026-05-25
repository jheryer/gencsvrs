mod util;
use std::error::Error;
use std::path::PathBuf;
use util::schema::{default_schema, parse_schema};
use util::{dataframe::create_dataframe, output::Console};

use crate::util::output::{CSVFile, Output, ParquetFile};
use crate::util::scanner;
type RunResult<T> = Result<T, Box<dyn Error>>;

/// Output format selector for the `er` subcommand. M1 only parses the flag;
/// the actual sink wiring lands in M3.
#[derive(Clone, Copy, Debug)]
pub enum ErFormat {
    Csv,
    Parquet,
}

/// M1 entry point for the `gencsv er <FILE>` subcommand. Reads the Mermaid
/// source, runs the scanner, and dumps the token stream to stdout. Parser and
/// generator land in M2/M3 — the `rows`, `rows_per`, `out`, `format` args are
/// parsed here so the CLI surface is stable, but they have no effect yet.
pub fn run_er(
    file: &str,
    rows: usize,
    rows_per: Vec<(String, usize)>,
    out: PathBuf,
    format: ErFormat,
) -> RunResult<()> {
    let _ = (rows, rows_per, out, format); // reserved for M2/M3

    let contents = std::fs::read_to_string(file)
        .map_err(|e| format!("failed to read ER source '{file}': {e}"))?;

    let tokens =
        scanner::scan(&contents).map_err(|e| format!("{file}:{}: {}", e.line, e.message))?;

    for (line, tok) in tokens {
        println!("{line}\t{tok:?}");
    }

    Ok(())
}

pub fn run(
    schema: Option<String>,
    rows: usize,
    file_target: Option<String>,
    csv: bool,
    parquet: bool,
    append_target: Option<String>,
    delete_target: Option<String>,
) -> RunResult<()> {
    // Default to CSV when neither output flag is set.
    let csv = csv || !parquet;

    // Parquet output requires a target file; otherwise data would be silently discarded.
    if parquet && file_target.is_none() {
        return Err(
            "--parquet output requires --file-target <PATH>; refusing to discard generated rows"
                .into(),
        );
    }

    let tokenized_schema = match schema {
        Some(ref s) => {
            let parsed = parse_schema(s.as_str());
            if parsed.is_empty() {
                return Err(format!(
                    "schema string '{s}' produced no valid columns; expected 'name:TYPE[,name:TYPE...]'"
                )
                .into());
            }
            parsed
        }
        None => default_schema(),
    };

    let mut data_frame = create_dataframe(tokenized_schema, rows, append_target, delete_target)
        .map_err(|e| format!("failed to build dataframe: {e}"))?;

    match (csv, parquet, file_target) {
        (_, true, Some(path)) => ParquetFile { file_name: path }.write(&mut data_frame)?,
        (true, _, Some(path)) => CSVFile { file_name: path }.write(&mut data_frame)?,
        _ => Console {}.write(&mut data_frame)?,
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parquet_without_file_target_is_rejected() {
        // Previously: silently generated and discarded data, exit 0.
        // Now: must surface a clear error to the user.
        let result = run(
            Some("a:INT,b:STRING".to_string()),
            3,
            None,  // no -f / --file-target
            false, // -c
            true,  // -p
            None,
            None,
        );
        assert!(
            result.is_err(),
            "expected Err when --parquet is set without --file-target"
        );
    }

    #[test]
    fn empty_schema_returns_descriptive_error() {
        let result = run(Some("".to_string()), 3, None, true, false, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn run_csv_to_file_succeeds() {
        let path = std::env::temp_dir().join("gencsv_lib_test_csv.csv");
        let result = run(
            Some("id:INT_INC,name:VALUE".to_string()),
            5,
            Some(path.to_str().unwrap().to_string()),
            true,
            false,
            None,
            None,
        );
        assert!(result.is_ok());
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn run_parquet_to_file_succeeds() {
        let path = std::env::temp_dir().join("gencsv_lib_test_parquet.parquet");
        let result = run(
            Some("id:INT_INC,name:VALUE".to_string()),
            5,
            Some(path.to_str().unwrap().to_string()),
            false,
            true,
            None,
            None,
        );
        assert!(result.is_ok());
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn run_bad_append_target_returns_error() {
        let result = run(
            Some("id:INT_INC".to_string()),
            3,
            None,
            true,
            false,
            Some("/nonexistent/file.parquet".to_string()),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn run_default_schema_succeeds() {
        let result = run(None, 5, None, true, false, None, None);
        assert!(result.is_ok());
    }
}
