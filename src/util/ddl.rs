//! DDL emitter for D2 of `.claude/prds/db-target-types.prd.md`.
//!
//! Produces a `CREATE TABLE` statement from a gencsv flat-table schema and a
//! target dialect. The first `INT_INC` column is treated as the primary key;
//! subsequent `INT_INC` columns are non-PK. FK constraints are deferred to D5
//! where the ER AST provides the relational context.

use crate::util::dialect::{to_sql_type, Dialect, DialectError};
use crate::util::schema::Schema;

/// Emit a `CREATE TABLE` DDL string for `table_name` using `columns` and
/// `dialect`. Returns an error only if a column's type has no mapping for the
/// chosen dialect (i.e. `to_sql_type` fails).
pub fn emit_create_table(
    table_name: &str,
    columns: &[Schema],
    dialect: Dialect,
) -> Result<String, DialectError> {
    let mut pk_seen = false;
    let mut col_defs: Vec<String> = Vec::with_capacity(columns.len());

    for col in columns {
        let is_pk = col.datatype == "INT_INC" && !pk_seen;
        if is_pk {
            pk_seen = true;
        }
        let sql_type = to_sql_type(&col.datatype, dialect, is_pk)?;
        col_defs.push(format!("  {} {}", col.name, sql_type));
    }

    Ok(format!(
        "CREATE TABLE {} (\n{}\n);\n",
        table_name,
        col_defs.join(",\n")
    ))
}

/// Derive the DDL output path from a data file path.
///
/// `data_path` is the `--file-target` value (e.g. `"./out/users.csv"`).
/// Returns e.g. `"./out/users.ddl.postgres.sql"`.
pub fn ddl_path(data_path: &str, dialect: Dialect) -> String {
    let stem = data_path
        .strip_suffix(".csv")
        .or_else(|| data_path.strip_suffix(".parquet"))
        .unwrap_or(data_path);
    format!("{}.ddl.{}.sql", stem, dialect.as_str())
}

/// Derive the table name from a data file path (basename without extension).
pub fn table_name_from_path(data_path: &str) -> &str {
    let path = std::path::Path::new(data_path);
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(data_path)
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;
    use crate::util::dialect::Dialect;
    use crate::util::schema::Schema;

    fn col(name: &str, datatype: &str) -> Schema {
        Schema {
            name: name.into(),
            datatype: datatype.into(),
            modifier: None,
        }
    }

    #[test]
    fn postgres_create_table_has_correct_shape() {
        let cols = vec![col("id", "INT_INC"), col("name", "STRING")];
        let ddl = emit_create_table("users", &cols, Dialect::Postgres).unwrap();
        assert!(ddl.starts_with("CREATE TABLE users ("), "got: {ddl}");
        assert!(ddl.contains("id SERIAL PRIMARY KEY"), "got: {ddl}");
        assert!(ddl.contains("name TEXT"), "got: {ddl}");
        assert!(ddl.ends_with(");\n"), "got: {ddl}");
    }

    #[test]
    fn mysql_uses_auto_increment_for_first_int_inc() {
        let cols = vec![col("id", "INT_INC"), col("seq", "INT_INC")];
        let ddl = emit_create_table("t", &cols, Dialect::Mysql).unwrap();
        assert!(
            ddl.contains("id INT AUTO_INCREMENT PRIMARY KEY"),
            "got: {ddl}"
        );
        assert!(ddl.contains("seq INT NOT NULL"), "got: {ddl}");
    }

    #[test]
    fn sqlserver_uses_identity_for_pk() {
        let cols = vec![col("id", "INT_INC"), col("email", "STRING")];
        let ddl = emit_create_table("accts", &cols, Dialect::Sqlserver).unwrap();
        assert!(
            ddl.contains("id INT IDENTITY(1,1) PRIMARY KEY"),
            "got: {ddl}"
        );
        assert!(ddl.contains("email NVARCHAR(255)"), "got: {ddl}");
    }

    #[test]
    fn bigquery_no_native_pk_decoration() {
        let cols = vec![col("id", "INT_INC"), col("price", "DECIMAL")];
        let ddl = emit_create_table("orders", &cols, Dialect::Bigquery).unwrap();
        assert!(ddl.contains("id INT64"), "got: {ddl}");
        assert!(ddl.contains("price NUMERIC"), "got: {ddl}");
    }

    #[test]
    fn all_types_round_trip_without_error() {
        let types = [
            "INT_INC",
            "INT",
            "INT_RNG",
            "DIGIT",
            "DECIMAL",
            "PRICE",
            "STRING",
            "VALUE",
            "DATE",
            "TIME",
            "DATE_TIME",
            "NAME",
            "FIRST_NAME",
            "LAST_NAME",
            "SSN",
            "ZIP_CODE",
            "COUNTRY_CODE",
            "STATE_NAME",
            "STATE_ABBR",
            "LAT",
            "LON",
            "PHONE",
            "LOREM_WORD",
            "LOREM_TITLE",
            "LOREM_SENTENCE",
            "LOREM_PARAGRAPH",
            "UUID",
        ];
        for dialect in [
            Dialect::Mysql,
            Dialect::Postgres,
            Dialect::Sqlserver,
            Dialect::Bigquery,
            Dialect::Spark,
        ] {
            let cols: Vec<Schema> = types.iter().map(|t| col(t, t)).collect();
            let result = emit_create_table("all_types", &cols, dialect);
            assert!(result.is_ok(), "dialect={dialect:?}: {result:?}");
        }
    }

    #[test]
    fn ddl_path_strips_csv_extension() {
        assert_eq!(
            ddl_path("./out/users.csv", Dialect::Postgres),
            "./out/users.ddl.postgres.sql"
        );
    }

    #[test]
    fn ddl_path_strips_parquet_extension() {
        assert_eq!(
            ddl_path("/data/orders.parquet", Dialect::Mysql),
            "/data/orders.ddl.mysql.sql"
        );
    }

    #[test]
    fn ddl_path_appends_when_no_known_extension() {
        assert_eq!(ddl_path("output", Dialect::Spark), "output.ddl.spark.sql");
    }

    #[test]
    fn table_name_from_path_strips_dir_and_extension() {
        assert_eq!(table_name_from_path("./out/users.csv"), "users");
        assert_eq!(table_name_from_path("/data/orders.parquet"), "orders");
        assert_eq!(table_name_from_path("plain"), "plain");
    }
}
