//! Load-command emitter for D3 of `.claude/prds/db-target-types.prd.md`.
//!
//! Produces a dialect-specific load snippet (SQL, shell, or Python) that the
//! user can run to import generated data into a target database. Templates
//! match PRD §6.3 exactly.

use crate::util::dialect::Dialect;

/// File extension for the load-command output file per dialect.
pub fn load_cmd_ext(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Mysql | Dialect::Postgres | Dialect::Sqlserver => "sql",
        Dialect::Bigquery => "sh",
        Dialect::Spark => "py",
    }
}

/// Derive the load-command output path from a data file path.
///
/// E.g. `"./out/users.csv"` + Postgres → `"./out/users.load.postgres.sql"`.
pub fn load_cmd_path(data_path: &str, dialect: Dialect) -> String {
    let stem = data_path
        .strip_suffix(".csv")
        .or_else(|| data_path.strip_suffix(".parquet"))
        .unwrap_or(data_path);
    format!(
        "{}.load.{}.{}",
        stem,
        dialect.as_str(),
        load_cmd_ext(dialect)
    )
}

/// Emit the load command for `table` loading from `file`.
///
/// `is_parquet` switches BigQuery and Spark templates between CSV and Parquet
/// variants. MySQL, Postgres, and SQL Server always use CSV templates.
pub fn emit_load_cmd(table: &str, file: &str, dialect: Dialect, is_parquet: bool) -> String {
    match dialect {
        Dialect::Mysql => format!(
            "LOAD DATA LOCAL INFILE '{}' INTO TABLE {} \
             FIELDS TERMINATED BY ',' ENCLOSED BY '\"' \
             LINES TERMINATED BY '\\n' IGNORE 1 ROWS;\n",
            file, table
        ),
        Dialect::Postgres => format!(
            "\\copy {} FROM '{}' WITH (FORMAT csv, HEADER true);\n",
            table, file
        ),
        Dialect::Sqlserver => format!(
            "BULK INSERT {} FROM '{}' WITH (FORMAT = 'CSV', FIRSTROW = 2, KEEPIDENTITY);\n",
            table, file
        ),
        Dialect::Bigquery => {
            if is_parquet {
                format!(
                    "bq load --source_format=PARQUET dataset.{} {}\n",
                    table, file
                )
            } else {
                format!(
                    "bq load --source_format=CSV --skip_leading_rows=1 dataset.{} {}\n",
                    table, file
                )
            }
        }
        Dialect::Spark => {
            if is_parquet {
                format!(
                    "spark.read.parquet(\"{}\").write.saveAsTable(\"{}\")\n",
                    file, table
                )
            } else {
                format!(
                    "spark.read.option(\"header\", True).csv(\"{}\").write.saveAsTable(\"{}\")\n",
                    file, table
                )
            }
        }
    }
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;

    #[test]
    fn mysql_csv_load_command() {
        let cmd = emit_load_cmd("users", "users.csv", Dialect::Mysql, false);
        assert!(
            cmd.contains("LOAD DATA LOCAL INFILE 'users.csv'"),
            "got: {cmd}"
        );
        assert!(cmd.contains("INTO TABLE users"), "got: {cmd}");
        assert!(cmd.contains("IGNORE 1 ROWS"), "got: {cmd}");
    }

    #[test]
    fn postgres_csv_load_command() {
        let cmd = emit_load_cmd("orders", "orders.csv", Dialect::Postgres, false);
        assert!(cmd.contains("\\copy orders"), "got: {cmd}");
        assert!(cmd.contains("FORMAT csv, HEADER true"), "got: {cmd}");
    }

    #[test]
    fn sqlserver_csv_load_command() {
        let cmd = emit_load_cmd("t", "t.csv", Dialect::Sqlserver, false);
        assert!(cmd.contains("BULK INSERT t"), "got: {cmd}");
        assert!(cmd.contains("FIRSTROW = 2"), "got: {cmd}");
    }

    #[test]
    fn bigquery_csv_load_command() {
        let cmd = emit_load_cmd("events", "events.csv", Dialect::Bigquery, false);
        assert!(cmd.contains("--source_format=CSV"), "got: {cmd}");
        assert!(cmd.contains("--skip_leading_rows=1"), "got: {cmd}");
        assert!(cmd.contains("dataset.events"), "got: {cmd}");
    }

    #[test]
    fn bigquery_parquet_load_command() {
        let cmd = emit_load_cmd("events", "events.parquet", Dialect::Bigquery, true);
        assert!(cmd.contains("--source_format=PARQUET"), "got: {cmd}");
        assert!(!cmd.contains("skip_leading_rows"), "got: {cmd}");
    }

    #[test]
    fn spark_csv_load_command() {
        let cmd = emit_load_cmd("logs", "logs.csv", Dialect::Spark, false);
        assert!(
            cmd.contains("option(\"header\", True).csv(\"logs.csv\")"),
            "got: {cmd}"
        );
        assert!(cmd.contains("saveAsTable(\"logs\")"), "got: {cmd}");
    }

    #[test]
    fn spark_parquet_load_command() {
        let cmd = emit_load_cmd("logs", "logs.parquet", Dialect::Spark, true);
        assert!(cmd.contains("read.parquet(\"logs.parquet\")"), "got: {cmd}");
        assert!(cmd.contains("saveAsTable(\"logs\")"), "got: {cmd}");
    }

    #[test]
    fn load_cmd_path_derives_correct_extension() {
        assert_eq!(
            load_cmd_path("users.csv", Dialect::Postgres),
            "users.load.postgres.sql"
        );
        assert_eq!(
            load_cmd_path("events.csv", Dialect::Bigquery),
            "events.load.bigquery.sh"
        );
        assert_eq!(
            load_cmd_path("logs.parquet", Dialect::Spark),
            "logs.load.spark.py"
        );
        assert_eq!(load_cmd_path("t.csv", Dialect::Mysql), "t.load.mysql.sql");
    }

    #[test]
    fn load_cmd_ext_per_dialect() {
        assert_eq!(load_cmd_ext(Dialect::Mysql), "sql");
        assert_eq!(load_cmd_ext(Dialect::Postgres), "sql");
        assert_eq!(load_cmd_ext(Dialect::Sqlserver), "sql");
        assert_eq!(load_cmd_ext(Dialect::Bigquery), "sh");
        assert_eq!(load_cmd_ext(Dialect::Spark), "py");
    }
}
