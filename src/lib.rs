mod util;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use util::schema::{default_schema, parse_schema};
use util::{dataframe::create_dataframe, output::Console};

use crate::util::ddl::{ddl_path, emit_create_table, emit_er_ddl, table_name_from_path};
use crate::util::generator::generate;
use crate::util::load_cmd::{emit_load_cmd, load_cmd_path};
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

/// Entry point for the `gencsv er <FILE>` subcommand.
#[allow(clippy::too_many_arguments)]
pub fn run_er(
    file: &str,
    rows: usize,
    rows_per: Vec<(String, usize)>,
    out: PathBuf,
    format: ErFormat,
    target: Option<Dialect>,
    no_ddl: bool,
    no_load: bool,
) -> RunResult<()> {
    let is_parquet = matches!(format, ErFormat::Parquet);

    let contents = std::fs::read_to_string(file)
        .map_err(|e| format!("failed to read ER source '{file}': {e}"))?;

    let tokens = scan_erd(&contents).map_err(|e| format!("{file}:{}: {}", e.line, e.message))?;
    let ast = parse_erd(tokens).map_err(|e| format!("{file}: {}", e.message))?;

    // D4: warn when BigQuery/Spark + Parquet (logical-type alignment is documented, not auto-cast)
    if let Some(d) = target {
        if is_parquet && matches!(d, Dialect::Bigquery | Dialect::Spark) {
            eprintln!(
                "warning: --target {} with --format parquet: \
                 review docs/DIALECTS.md for recommended Parquet logical types",
                d.as_str()
            );
        }
    }

    let rows_per_map: HashMap<String, usize> = rows_per.into_iter().collect();
    let frames = generate(&ast, rows, &rows_per_map).map_err(|e| e.message)?;
    let ordered_names: Vec<String> = frames.iter().map(|(n, _)| n.clone()).collect();

    let sink = MultiFileSink::new(out.clone(), format.into())?;
    for (name, mut df) in frames {
        let path = sink.write(&name, &mut df)?;
        eprintln!("wrote {}", path.display());

        if let Some(dialect) = target {
            let path_str = path.to_str().ok_or("output path is not valid UTF-8")?;
            let table = name.as_str();

            if !no_ddl {
                // Per-entity DDL is emitted as part of the combined ER DDL below.
                // Individual entity DDL not separately emitted here.
            }

            if !no_load {
                let load = emit_load_cmd(table, path_str, dialect, is_parquet);
                let load_path = load_cmd_path(path_str, dialect);
                std::fs::write(&load_path, &load)
                    .map_err(|e| format!("failed to write load command '{load_path}': {e}"))?;
                eprintln!("wrote {load_path}");
            }
        }
    }

    // Emit combined DDL file for all entities in topological order (D5)
    if let Some(dialect) = target {
        if !no_ddl {
            let ddl = emit_er_ddl(&ast, &ordered_names, dialect)
                .map_err(|e| format!("DDL emit failed: {e}"))?;
            let ddl_file = out.join(format!("schema.ddl.{}.sql", dialect.as_str()));
            std::fs::write(&ddl_file, &ddl)
                .map_err(|e| format!("failed to write DDL '{}: {e}", ddl_file.display()))?;
            eprintln!("wrote {}", ddl_file.display());
        }
    }

    Ok(())
}

// Consolidate into a RunOptions struct when D3 flags stabilise.
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
    no_load: bool,
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

    // D4: warn when BigQuery/Spark + Parquet
    if let Some(d) = target {
        if parquet && matches!(d, Dialect::Bigquery | Dialect::Spark) {
            eprintln!(
                "warning: --target {} with --parquet: \
                 review docs/DIALECTS.md for recommended Parquet logical types",
                d.as_str()
            );
        }
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

    if let Some(dialect) = target {
        let path = file_target.as_ref().expect("guarded above");
        let table = table_name_from_path(path);

        if !no_ddl {
            let ddl = emit_create_table(table, &tokenized_schema, dialect)
                .map_err(|e| format!("DDL emit failed: {e}"))?;
            let out_path = ddl_path(path, dialect);
            std::fs::write(&out_path, &ddl)
                .map_err(|e| format!("failed to write DDL file '{out_path}': {e}"))?;
            eprintln!("wrote {out_path}");
        }

        if !no_load {
            let load = emit_load_cmd(table, path, dialect, parquet);
            let out_path = load_cmd_path(path, dialect);
            std::fs::write(&out_path, &load)
                .map_err(|e| format!("failed to write load command '{out_path}': {e}"))?;
            eprintln!("wrote {out_path}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn run9(
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
        run(
            schema,
            rows,
            file_target,
            csv,
            parquet,
            append_target,
            delete_target,
            target,
            no_ddl,
            false,
        )
    }

    #[test]
    fn parquet_without_file_target_is_rejected() {
        let result = run9(
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
        let result = run9(
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
        let result = run9(
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
        let result = run9(
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
        let result = run9(
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
        let result = run9(None, 5, None, true, false, None, None, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn target_without_file_target_is_rejected() {
        let result = run9(
            Some("id:INT_INC".to_string()),
            3,
            None,
            true,
            false,
            None,
            None,
            Some(Dialect::Mysql),
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("--target requires --file-target"),
            "got: {msg}"
        );
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
        let r = run_er(
            mmd.to_str().unwrap(),
            5,
            vec![],
            dir.clone(),
            ErFormat::Csv,
            None,
            false,
            false,
        );
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
        let r = run_er(
            mmd.to_str().unwrap(),
            5,
            vec![],
            dir.clone(),
            ErFormat::Csv,
            None,
            false,
            false,
        );
        assert!(r.is_ok(), "run_er failed: {r:?}");
        assert!(dir.join("STUDENT.csv").exists());
        assert!(dir.join("COURSE.csv").exists());
        assert!(dir.join("STUDENT_COURSE.csv").exists());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&mmd);
    }

    #[test]
    fn run_er_target_postgres_writes_ddl_and_load() {
        let src = "\
erDiagram
  CUSTOMER { int id PK }
  ORDER { int id PK }
  CUSTOMER ||--o{ ORDER : places
";
        let dir = std::env::temp_dir().join("gencsv_run_er_ddl_test");
        let _ = std::fs::remove_dir_all(&dir);
        let mmd = std::env::temp_dir().join("gencsv_run_er_ddl_test.mmd");
        std::fs::write(&mmd, src).unwrap();
        let r = run_er(
            mmd.to_str().unwrap(),
            5,
            vec![],
            dir.clone(),
            ErFormat::Csv,
            Some(Dialect::Postgres),
            false,
            false,
        );
        assert!(r.is_ok(), "run_er failed: {r:?}");
        assert!(dir.join("schema.ddl.postgres.sql").exists(), "DDL missing");
        assert!(
            dir.join("CUSTOMER.load.postgres.sql").exists(),
            "load cmd missing"
        );
        let ddl = std::fs::read_to_string(dir.join("schema.ddl.postgres.sql")).unwrap();
        assert!(ddl.contains("CREATE TABLE CUSTOMER"), "got: {ddl}");
        assert!(ddl.contains("CREATE TABLE ORDER"), "got: {ddl}");
        assert!(ddl.contains("FOREIGN KEY"), "got: {ddl}");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&mmd);
    }
}
