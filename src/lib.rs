mod util;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use util::schema::{default_schema, parse_schema};
use util::{dataframe::create_dataframe, output::Console};

use crate::util::ddl::{ddl_path, emit_create_table, table_name_from_path};
use crate::util::generator::generate;
use crate::util::multi_file_sink::{MultiFileSink, SinkFormat};
use crate::util::output::{CSVFile, Output, ParquetFile};
use crate::util::parser::parse as parse_erd;
use crate::util::scanner::scan as scan_erd;
type RunResult<T> = Result<T, Box<dyn Error>>;

pub use util::dialect::{to_sql_type, Dialect, DialectError};

/// Output format selector for the `er` subcommand.
#[derive(Clone, Copy, Debug)]
pub enum ErFormat {
    Csv,
    Parquet,
}

impl From<ErFormat> for SinkFormat {
    fn from(f: ErFormat) -> Self {
        match f {
            ErFormat::Csv => SinkFormat::Csv,
            ErFormat::Parquet => SinkFormat::Parquet,
        }
    }
}

/// Entry point for the `gencsv er <FILE>` subcommand. Scans, parses,
/// validates, generates, and writes one file per entity (plus one per M:N
/// junction) under `out`.
pub fn run_er(
    file: &str,
    rows: usize,
    rows_per: Vec<(String, usize)>,
    out: PathBuf,
    format: ErFormat,
) -> RunResult<()> {
    let contents = std::fs::read_to_string(file)
        .map_err(|e| format!("failed to read ER source '{file}': {e}"))?;

    let tokens = scan_erd(&contents).map_err(|e| format!("{file}:{}: {}", e.line, e.message))?;

    let ast = parse_erd(tokens).map_err(|e| format!("{file}: {}", e.message))?;

    let rows_per_map: HashMap<String, usize> = rows_per.into_iter().collect();
    let frames = generate(&ast, rows, &rows_per_map).map_err(|e| e.message)?;

    let sink = MultiFileSink::new(out, format.into())?;
    for (name, mut df) in frames {
        let path = sink.write(&name, &mut df)?;
        eprintln!("wrote {}", path.display());
    }

    Ok(())
}

// D3 will add --no-load and --no-data; consolidate into a struct then.
#[allow(clippy::too_many_arguments)]
pub fn run(
    schema: Option<String>,
    rows: usize,
    file_target: Option<String>,
    csv: bool,
    parquet: bool,
    append_target: Option<String>,
    delete_target: Option<String>,
    target: Option<Dialect>,
    no_ddl: bool,
) -> RunResult<()> {
    let csv = csv || !parquet;

    if parquet && file_target.is_none() {
        return Err(
            "--parquet output requires --file-target <PATH>; refusing to discard generated rows"
                .into(),
        );
    }

    if target.is_some() && file_target.is_none() {
        return Err(
            "--target requires --file-target so the DDL file can be placed next to the data".into(),
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

    let mut data_frame =
        create_dataframe(tokenized_schema.clone(), rows, append_target, delete_target)
            .map_err(|e| format!("failed to build dataframe: {e}"))?;

    match (csv, parquet, &file_target) {
        (_, true, Some(path)) => ParquetFile {
            file_name: path.clone(),
        }
        .write(&mut data_frame)?,
        (true, _, Some(path)) => CSVFile {
            file_name: path.clone(),
        }
        .write(&mut data_frame)?,
        _ => Console {}.write(&mut data_frame)?,
    }

    if let (Some(dialect), Some(ref path), false) = (target, &file_target, no_ddl) {
        let table = table_name_from_path(path);
        let ddl = emit_create_table(table, &tokenized_schema, dialect)
            .map_err(|e| format!("DDL emit failed: {e}"))?;
        let out_path = ddl_path(path, dialect);
        std::fs::write(&out_path, &ddl)
            .map_err(|e| format!("failed to write DDL file '{out_path}': {e}"))?;
        eprintln!("wrote {out_path}");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parquet_without_file_target_is_rejected() {
        let result = run(
            Some("a:INT,b:STRING".to_string()),
            3,
            None,
            false,
            true,
            None,
            None,
            None,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn empty_schema_returns_descriptive_error() {
        let result = run(
            Some("".to_string()),
            3,
            None,
            true,
            false,
            None,
            None,
            None,
            false,
        );
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
            None,
            false,
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
            None,
            false,
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
            None,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn run_default_schema_succeeds() {
        let result = run(None, 5, None, true, false, None, None, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn run_er_writes_files_per_entity() {
        let src = "\
erDiagram
  PARENT { int id PK }
  CHILD { int id PK }
  PARENT ||--o{ CHILD : has
";
        let dir = std::env::temp_dir().join("gencsv_run_er_test_basic");
        let _ = std::fs::remove_dir_all(&dir);
        let mmd = std::env::temp_dir().join("gencsv_run_er_test_basic.mmd");
        std::fs::write(&mmd, src).unwrap();
        let r = run_er(mmd.to_str().unwrap(), 5, vec![], dir.clone(), ErFormat::Csv);
        assert!(r.is_ok(), "run_er failed: {r:?}");
        assert!(dir.join("PARENT.csv").exists());
        assert!(dir.join("CHILD.csv").exists());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&mmd);
    }

    #[test]
    fn run_er_emits_junction_for_many_to_many() {
        let src = "\
erDiagram
  STUDENT { int id PK }
  COURSE { int id PK }
  STUDENT }o--o{ COURSE : enrolled
";
        let dir = std::env::temp_dir().join("gencsv_run_er_test_mn");
        let _ = std::fs::remove_dir_all(&dir);
        let mmd = std::env::temp_dir().join("gencsv_run_er_test_mn.mmd");
        std::fs::write(&mmd, src).unwrap();
        let r = run_er(mmd.to_str().unwrap(), 5, vec![], dir.clone(), ErFormat::Csv);
        assert!(r.is_ok(), "run_er failed: {r:?}");
        assert!(dir.join("STUDENT.csv").exists());
        assert!(dir.join("COURSE.csv").exists());
        assert!(dir.join("STUDENT_COURSE.csv").exists());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&mmd);
    }
}
