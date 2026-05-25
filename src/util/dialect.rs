//! Database-target dialect support (D1 of `.claude/prds/db-target-types.prd.md`).
//!
//! This module exposes:
//! - `Dialect`: the set of supported target databases
//! - `to_sql_type`: mapping from a gencsv schema type (uppercase keys matching
//!   `src/util/fake.rs::create_column`) to a dialect-specific SQL column type
//!   string per PRD §6.2.
//!
//! D1 is the lookup table only. DDL string assembly, load commands, Parquet
//! logical-type wiring, and the `--target` CLI flag land in D2+.

use clap::ValueEnum;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Dialect {
    Mysql,
    Postgres,
    Sqlserver,
    Bigquery,
    Spark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialectError {
    pub message: String,
}

impl fmt::Display for DialectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for DialectError {}

impl Dialect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dialect::Mysql => "mysql",
            Dialect::Postgres => "postgres",
            Dialect::Sqlserver => "sqlserver",
            Dialect::Bigquery => "bigquery",
            Dialect::Spark => "spark",
        }
    }

    /// Parse a `--target` CLI argument. Case-insensitive so users don't need
    /// to remember exact casing. Inherent method (not `std::str::FromStr`)
    /// so the error type stays local to this module — the trait's associated
    /// `Err` type would force a more awkward signature.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, DialectError> {
        match s.to_lowercase().as_str() {
            "mysql" => Ok(Dialect::Mysql),
            "postgres" => Ok(Dialect::Postgres),
            "sqlserver" => Ok(Dialect::Sqlserver),
            "bigquery" => Ok(Dialect::Bigquery),
            "spark" => Ok(Dialect::Spark),
            _ => Err(DialectError {
                message: format!(
                    "unsupported target '{s}'; supported: mysql, postgres, sqlserver, bigquery, spark"
                ),
            }),
        }
    }
}

/// Map a gencsv schema type to a SQL column-type string for the given dialect.
///
/// `gencsv_type` must be one of the uppercase keys accepted by
/// `src/util/fake.rs::create_column` (e.g. `"INT_INC"`, `"STRING"`,
/// `"DATE_TIME"`). `is_pk` only changes the result for `"INT_INC"`; for every
/// other type it is accepted but ignored. PK decoration of non-`INT_INC`
/// columns is deferred to D2 where the DDL emitter has full column context.
pub fn to_sql_type(
    gencsv_type: &str,
    dialect: Dialect,
    is_pk: bool,
) -> Result<String, DialectError> {
    use Dialect::*;
    let mapped: &str = match (gencsv_type, dialect) {
        // INT_INC: only type whose mapping depends on is_pk
        ("INT_INC", Mysql) if is_pk => "INT AUTO_INCREMENT PRIMARY KEY",
        ("INT_INC", Postgres) if is_pk => "SERIAL PRIMARY KEY",
        ("INT_INC", Sqlserver) if is_pk => "INT IDENTITY(1,1) PRIMARY KEY",
        ("INT_INC", Bigquery) if is_pk => "INT64",
        ("INT_INC", Spark) if is_pk => "INT",
        ("INT_INC", Mysql) => "INT NOT NULL",
        ("INT_INC", Postgres) => "INTEGER NOT NULL",
        ("INT_INC", Sqlserver) => "INT NOT NULL",
        ("INT_INC", Bigquery) => "INT64",
        ("INT_INC", Spark) => "INT",

        // INT, INT_RNG: plain integers
        ("INT", Mysql) | ("INT_RNG", Mysql) => "INT",
        ("INT", Postgres) | ("INT_RNG", Postgres) => "INTEGER",
        ("INT", Sqlserver) | ("INT_RNG", Sqlserver) => "INT",
        ("INT", Bigquery) | ("INT_RNG", Bigquery) => "INT64",
        ("INT", Spark) | ("INT_RNG", Spark) => "INT",

        // DIGIT: single character
        ("DIGIT", Mysql) | ("DIGIT", Postgres) | ("DIGIT", Sqlserver) => "CHAR(1)",
        ("DIGIT", Bigquery) | ("DIGIT", Spark) => "STRING",

        // DECIMAL
        ("DECIMAL", Mysql) => "DECIMAL(10,2)",
        ("DECIMAL", Postgres) => "NUMERIC(10,2)",
        ("DECIMAL", Sqlserver) => "DECIMAL(10,2)",
        ("DECIMAL", Bigquery) => "NUMERIC",
        ("DECIMAL", Spark) => "DECIMAL(10,2)",

        // PRICE: SQL Server gets MONEY
        ("PRICE", Mysql) => "DECIMAL(10,2)",
        ("PRICE", Postgres) => "NUMERIC(10,2)",
        ("PRICE", Sqlserver) => "MONEY",
        ("PRICE", Bigquery) => "NUMERIC",
        ("PRICE", Spark) => "DECIMAL(10,2)",

        // STRING: longest variable text
        ("STRING", Mysql) => "VARCHAR(255)",
        ("STRING", Postgres) => "TEXT",
        ("STRING", Sqlserver) => "NVARCHAR(255)",
        ("STRING", Bigquery) | ("STRING", Spark) => "STRING",

        // VALUE: short fixed-purpose literal
        ("VALUE", Mysql) | ("VALUE", Postgres) | ("VALUE", Sqlserver) => "VARCHAR(50)",
        ("VALUE", Bigquery) | ("VALUE", Spark) => "STRING",

        // DATE / TIME / DATE_TIME
        ("DATE", Mysql) | ("DATE", Postgres) | ("DATE", Sqlserver) => "DATE",
        ("DATE", Bigquery) | ("DATE", Spark) => "DATE",

        ("TIME", Mysql) | ("TIME", Postgres) | ("TIME", Sqlserver) => "TIME",
        ("TIME", Bigquery) => "TIME",
        ("TIME", Spark) => "STRING", // Spark has no TIME type

        ("DATE_TIME", Mysql) => "DATETIME",
        ("DATE_TIME", Postgres) => "TIMESTAMP",
        ("DATE_TIME", Sqlserver) => "DATETIME2",
        ("DATE_TIME", Bigquery) | ("DATE_TIME", Spark) => "TIMESTAMP",

        // Name family
        ("NAME", Mysql) | ("FIRST_NAME", Mysql) | ("LAST_NAME", Mysql) => "VARCHAR(100)",
        ("NAME", Postgres) | ("FIRST_NAME", Postgres) | ("LAST_NAME", Postgres) => "TEXT",
        ("NAME", Sqlserver) | ("FIRST_NAME", Sqlserver) | ("LAST_NAME", Sqlserver) => {
            "NVARCHAR(100)"
        }
        ("NAME", Bigquery)
        | ("FIRST_NAME", Bigquery)
        | ("LAST_NAME", Bigquery)
        | ("NAME", Spark)
        | ("FIRST_NAME", Spark)
        | ("LAST_NAME", Spark) => "STRING",

        // SSN
        ("SSN", Mysql) | ("SSN", Postgres) | ("SSN", Sqlserver) => "CHAR(11)",
        ("SSN", Bigquery) | ("SSN", Spark) => "STRING",

        // ZIP_CODE
        ("ZIP_CODE", Mysql) | ("ZIP_CODE", Postgres) | ("ZIP_CODE", Sqlserver) => "VARCHAR(10)",
        ("ZIP_CODE", Bigquery) | ("ZIP_CODE", Spark) => "STRING",

        // COUNTRY_CODE
        ("COUNTRY_CODE", Mysql) | ("COUNTRY_CODE", Postgres) | ("COUNTRY_CODE", Sqlserver) => {
            "CHAR(2)"
        }
        ("COUNTRY_CODE", Bigquery) | ("COUNTRY_CODE", Spark) => "STRING",

        // STATE_NAME
        ("STATE_NAME", Mysql) | ("STATE_NAME", Postgres) => "VARCHAR(64)",
        ("STATE_NAME", Sqlserver) => "NVARCHAR(64)",
        ("STATE_NAME", Bigquery) | ("STATE_NAME", Spark) => "STRING",

        // STATE_ABBR
        ("STATE_ABBR", Mysql) | ("STATE_ABBR", Postgres) | ("STATE_ABBR", Sqlserver) => "CHAR(2)",
        ("STATE_ABBR", Bigquery) | ("STATE_ABBR", Spark) => "STRING",

        // LAT / LON
        ("LAT", Mysql) | ("LON", Mysql) => "DECIMAL(9,6)",
        ("LAT", Postgres) | ("LON", Postgres) => "NUMERIC(9,6)",
        ("LAT", Sqlserver) | ("LON", Sqlserver) => "DECIMAL(9,6)",
        ("LAT", Bigquery) | ("LON", Bigquery) => "NUMERIC",
        ("LAT", Spark) | ("LON", Spark) => "DECIMAL(9,6)",

        // PHONE
        ("PHONE", Mysql) | ("PHONE", Postgres) | ("PHONE", Sqlserver) => "VARCHAR(20)",
        ("PHONE", Bigquery) | ("PHONE", Spark) => "STRING",

        // LOREM_WORD
        ("LOREM_WORD", Mysql) | ("LOREM_WORD", Postgres) => "VARCHAR(50)",
        ("LOREM_WORD", Sqlserver) => "NVARCHAR(50)",
        ("LOREM_WORD", Bigquery) | ("LOREM_WORD", Spark) => "STRING",

        // LOREM_TITLE
        ("LOREM_TITLE", Mysql) | ("LOREM_TITLE", Postgres) => "VARCHAR(200)",
        ("LOREM_TITLE", Sqlserver) => "NVARCHAR(200)",
        ("LOREM_TITLE", Bigquery) | ("LOREM_TITLE", Spark) => "STRING",

        // LOREM_SENTENCE / LOREM_PARAGRAPH
        ("LOREM_SENTENCE", Mysql)
        | ("LOREM_SENTENCE", Postgres)
        | ("LOREM_PARAGRAPH", Mysql)
        | ("LOREM_PARAGRAPH", Postgres) => "TEXT",
        ("LOREM_SENTENCE", Sqlserver) | ("LOREM_PARAGRAPH", Sqlserver) => "NVARCHAR(MAX)",
        ("LOREM_SENTENCE", Bigquery)
        | ("LOREM_SENTENCE", Spark)
        | ("LOREM_PARAGRAPH", Bigquery)
        | ("LOREM_PARAGRAPH", Spark) => "STRING",

        // UUID
        ("UUID", Mysql) => "CHAR(36)",
        ("UUID", Postgres) => "UUID",
        ("UUID", Sqlserver) => "UNIQUEIDENTIFIER",
        ("UUID", Bigquery) | ("UUID", Spark) => "STRING",

        (unknown, d) => {
            return Err(DialectError {
                message: format!(
                    "type '{unknown}' has no '{}' mapping; supported mappings: see docs/DIALECTS.md",
                    d.as_str()
                ),
            });
        }
    };
    Ok(mapped.to_string())
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;

    /// Canonical list of every gencsv type covered by D1, drawn from PRD §6.2.
    /// Used by the contract test in `every_type_has_mapping_for_every_dialect`.
    const SUPPORTED_TYPES: &[&str] = &[
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

    const ALL_DIALECTS: &[Dialect] = &[
        Dialect::Mysql,
        Dialect::Postgres,
        Dialect::Sqlserver,
        Dialect::Bigquery,
        Dialect::Spark,
    ];

    // ---------- from_str / as_str ----------

    #[test]
    fn from_str_round_trips_all_dialects() {
        for d in ALL_DIALECTS {
            let parsed = Dialect::from_str(d.as_str()).unwrap();
            assert_eq!(parsed, *d);
        }
    }

    #[test]
    fn from_str_rejects_unknown_target() {
        let err = Dialect::from_str("redshift").unwrap_err();
        assert!(
            err.message.contains("unsupported target 'redshift'"),
            "got: {}",
            err.message
        );
        for name in ["mysql", "postgres", "sqlserver", "bigquery", "spark"] {
            assert!(
                err.message.contains(name),
                "missing {name} in supported list: {}",
                err.message
            );
        }
    }

    #[test]
    fn from_str_is_case_insensitive() {
        assert_eq!(Dialect::from_str("MySQL").unwrap(), Dialect::Mysql);
        assert_eq!(Dialect::from_str("MYSQL").unwrap(), Dialect::Mysql);
        // 'PostgreSQL' isn't in the supported list — we accept 'postgres' only
        assert!(Dialect::from_str("PostgreSQL").is_err());
        assert_eq!(Dialect::from_str("BIGQUERY").unwrap(), Dialect::Bigquery);
    }

    #[test]
    fn dialect_display_is_lowercase() {
        assert_eq!(Dialect::Mysql.as_str(), "mysql");
        assert_eq!(Dialect::Postgres.as_str(), "postgres");
        assert_eq!(Dialect::Sqlserver.as_str(), "sqlserver");
        assert_eq!(Dialect::Bigquery.as_str(), "bigquery");
        assert_eq!(Dialect::Spark.as_str(), "spark");
    }

    // ---------- INT_INC (PK-sensitive) ----------

    #[test]
    fn int_inc_pk_maps_per_dialect() {
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Mysql, true).unwrap(),
            "INT AUTO_INCREMENT PRIMARY KEY"
        );
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Postgres, true).unwrap(),
            "SERIAL PRIMARY KEY"
        );
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Sqlserver, true).unwrap(),
            "INT IDENTITY(1,1) PRIMARY KEY"
        );
        assert!(to_sql_type("INT_INC", Dialect::Bigquery, true)
            .unwrap()
            .contains("INT64"));
        assert!(to_sql_type("INT_INC", Dialect::Spark, true)
            .unwrap()
            .contains("INT"));
    }

    #[test]
    fn int_inc_non_pk_drops_identity_decoration() {
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Mysql, false).unwrap(),
            "INT NOT NULL"
        );
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Postgres, false).unwrap(),
            "INTEGER NOT NULL"
        );
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Sqlserver, false).unwrap(),
            "INT NOT NULL"
        );
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Bigquery, false).unwrap(),
            "INT64"
        );
        assert_eq!(
            to_sql_type("INT_INC", Dialect::Spark, false).unwrap(),
            "INT"
        );
    }

    // ---------- Type-class spot checks (PRD §6.2 rows) ----------

    #[test]
    fn numeric_types_map_per_dialect() {
        assert_eq!(
            to_sql_type("INT", Dialect::Postgres, false).unwrap(),
            "INTEGER"
        );
        assert_eq!(
            to_sql_type("INT_RNG", Dialect::Bigquery, false).unwrap(),
            "INT64"
        );
        assert_eq!(
            to_sql_type("DECIMAL", Dialect::Postgres, false).unwrap(),
            "NUMERIC(10,2)"
        );
        assert_eq!(
            to_sql_type("PRICE", Dialect::Sqlserver, false).unwrap(),
            "MONEY"
        );
        assert_eq!(
            to_sql_type("LAT", Dialect::Mysql, false).unwrap(),
            "DECIMAL(9,6)"
        );
        assert_eq!(
            to_sql_type("LON", Dialect::Bigquery, false).unwrap(),
            "NUMERIC"
        );
    }

    #[test]
    fn string_types_map_per_dialect() {
        assert_eq!(
            to_sql_type("STRING", Dialect::Mysql, false).unwrap(),
            "VARCHAR(255)"
        );
        assert_eq!(
            to_sql_type("STRING", Dialect::Postgres, false).unwrap(),
            "TEXT"
        );
        assert_eq!(
            to_sql_type("STRING", Dialect::Sqlserver, false).unwrap(),
            "NVARCHAR(255)"
        );
        assert_eq!(
            to_sql_type("STRING", Dialect::Bigquery, false).unwrap(),
            "STRING"
        );
        assert_eq!(
            to_sql_type("VALUE", Dialect::Mysql, false).unwrap(),
            "VARCHAR(50)"
        );
        assert_eq!(
            to_sql_type("UUID", Dialect::Postgres, false).unwrap(),
            "UUID"
        );
        assert_eq!(
            to_sql_type("UUID", Dialect::Sqlserver, false).unwrap(),
            "UNIQUEIDENTIFIER"
        );
        assert_eq!(
            to_sql_type("UUID", Dialect::Mysql, false).unwrap(),
            "CHAR(36)"
        );
        assert_eq!(
            to_sql_type("SSN", Dialect::Mysql, false).unwrap(),
            "CHAR(11)"
        );
        assert_eq!(
            to_sql_type("COUNTRY_CODE", Dialect::Postgres, false).unwrap(),
            "CHAR(2)"
        );
        assert_eq!(
            to_sql_type("LOREM_PARAGRAPH", Dialect::Sqlserver, false).unwrap(),
            "NVARCHAR(MAX)"
        );
        assert_eq!(
            to_sql_type("NAME", Dialect::Sqlserver, false).unwrap(),
            "NVARCHAR(100)"
        );
    }

    #[test]
    fn date_time_types_map_per_dialect() {
        assert_eq!(to_sql_type("DATE", Dialect::Mysql, false).unwrap(), "DATE");
        assert_eq!(
            to_sql_type("TIME", Dialect::Postgres, false).unwrap(),
            "TIME"
        );
        // Spark has no TIME type — must fall back to STRING per PRD §6.2
        assert_eq!(
            to_sql_type("TIME", Dialect::Spark, false).unwrap(),
            "STRING"
        );
        assert_eq!(
            to_sql_type("DATE_TIME", Dialect::Mysql, false).unwrap(),
            "DATETIME"
        );
        assert_eq!(
            to_sql_type("DATE_TIME", Dialect::Postgres, false).unwrap(),
            "TIMESTAMP"
        );
        assert_eq!(
            to_sql_type("DATE_TIME", Dialect::Sqlserver, false).unwrap(),
            "DATETIME2"
        );
    }

    // ---------- Error paths ----------

    #[test]
    fn unknown_type_returns_error_with_dialect_name() {
        let err = to_sql_type("FOO", Dialect::Postgres, false).unwrap_err();
        assert!(err.message.contains("type 'FOO'"), "got: {}", err.message);
        assert!(err.message.contains("'postgres'"), "got: {}", err.message);
        assert!(
            err.message.contains("docs/DIALECTS.md"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn error_messages_quote_user_input_verbatim() {
        // Lowercase 'int' is not a recognized key — must NOT be silently
        // normalised to 'INT'. The error must echo what the user typed so
        // typos are easy to spot.
        let err = to_sql_type("int", Dialect::Postgres, false).unwrap_err();
        assert!(
            err.message.contains("'int'"),
            "expected verbatim 'int' in error: {}",
            err.message
        );
        assert!(
            !err.message.contains("'INT'"),
            "must not silently normalise to 'INT': {}",
            err.message
        );
    }

    // ---------- Contract invariants ----------

    #[test]
    fn is_pk_is_ignored_for_non_int_inc_types() {
        for d in ALL_DIALECTS {
            for t in SUPPORTED_TYPES {
                if *t == "INT_INC" {
                    continue;
                }
                let with_pk = to_sql_type(t, *d, true).unwrap();
                let without_pk = to_sql_type(t, *d, false).unwrap();
                assert_eq!(
                    with_pk, without_pk,
                    "is_pk changed mapping for ({t}, {:?})",
                    d
                );
            }
        }
    }

    #[test]
    fn every_type_has_mapping_for_every_dialect() {
        // The D1 contract: full coverage of PRD §6.2. If a type is added in
        // PRD §6.2, it MUST get a row in to_sql_type before this test passes.
        for d in ALL_DIALECTS {
            for t in SUPPORTED_TYPES {
                let result = to_sql_type(t, *d, false);
                assert!(
                    result.is_ok(),
                    "missing mapping for ({t}, {:?}): {:?}",
                    d,
                    result.err()
                );
                assert!(
                    !result.as_ref().unwrap().is_empty(),
                    "empty mapping for ({t}, {:?})",
                    d
                );
            }
        }
    }
}
