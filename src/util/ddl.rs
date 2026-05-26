//! DDL emitter (D2 + D5 of `.claude/prds/db-target-types.prd.md`).
//!
//! D2: flat-table `CREATE TABLE` from a synthtab schema.
//! D5: ER-mode `CREATE TABLE` with FK constraints, emitted in topological order.

use crate::util::dialect::{to_sql_type, Dialect, DialectError};
use crate::util::erd_ast::{ErdAst, KeyKind};
use crate::util::parser::mermaid_type_to_synthtab;
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

/// Emit DDL for all entities in `ordered_names` (topological order) plus any
/// M:N junction tables derived from the AST relationships.
///
/// Each table includes `FOREIGN KEY` constraints for MySQL, Postgres, and SQL
/// Server (BigQuery and Spark do not enforce FK constraints natively, so
/// constraints are omitted for those dialects).
pub fn emit_er_ddl(
    ast: &ErdAst,
    ordered_names: &[String],
    dialect: Dialect,
) -> Result<String, DialectError> {
    let mut out = String::new();
    let emit_fk_constraints = matches!(
        dialect,
        Dialect::Mysql | Dialect::Postgres | Dialect::Sqlserver
    );

    // Emit one CREATE TABLE per entity in topological order.
    for entity_name in ordered_names {
        let entity = ast.entity(entity_name).expect("entity from ordered list");
        let mut col_defs: Vec<String> = Vec::new();
        let mut fk_constraints: Vec<String> = Vec::new();

        // Declared columns
        for attr in &entity.attributes {
            let synthtab_type = mermaid_type_to_synthtab(&attr.data_type).unwrap_or("STRING");
            let is_pk = attr.key == Some(KeyKind::Pk);
            let sql_type = to_sql_type(synthtab_type, dialect, is_pk)?;
            col_defs.push(format!("  {} {}", attr.name, sql_type));
        }

        // FK columns and constraints from relationships where this entity is child
        for rel in &ast.relationships {
            if rel.cardinality.is_many_to_many() {
                continue;
            }
            if let Some((_parent, child)) = rel.cardinality.parent_child(&rel.left, &rel.right) {
                if child != entity_name {
                    continue;
                }
                let parent = if child == rel.left {
                    &rel.right
                } else {
                    &rel.left
                };
                let fk_col = format!("{}_id", parent.to_lowercase());
                // Only add FK column if not already declared by user
                let already_declared = entity.attributes.iter().any(|a| a.name == fk_col);
                if !already_declared {
                    col_defs.push(format!("  {} INTEGER", fk_col));
                }
                if emit_fk_constraints {
                    let parent_entity = ast.entity(parent).expect("parent in AST");
                    if let Some(pk) = parent_entity.pk() {
                        fk_constraints.push(format!(
                            "  CONSTRAINT fk_{}_{}_{} FOREIGN KEY ({}) REFERENCES {}({})",
                            entity_name.to_lowercase(),
                            parent.to_lowercase(),
                            pk.name,
                            fk_col,
                            parent,
                            pk.name
                        ));
                    }
                }
            }
        }

        let mut all_defs = col_defs;
        all_defs.extend(fk_constraints);
        out.push_str(&format!(
            "CREATE TABLE {} (\n{}\n);\n",
            entity_name,
            all_defs.join(",\n")
        ));
    }

    // Junction tables for M:N relationships
    for rel in &ast.relationships {
        if !rel.cardinality.is_many_to_many() {
            continue;
        }
        let junction = format!("{}_{}", rel.left, rel.right);
        let left_fk = format!("{}_id", rel.left.to_lowercase());
        let right_fk = format!("{}_id", rel.right.to_lowercase());
        let mut col_defs = vec![
            format!("  {} INTEGER", left_fk),
            format!("  {} INTEGER", right_fk),
        ];
        if emit_fk_constraints {
            let left_entity = ast.entity(&rel.left).expect("left entity in AST");
            let right_entity = ast.entity(&rel.right).expect("right entity in AST");
            if let (Some(lpk), Some(rpk)) = (left_entity.pk(), right_entity.pk()) {
                col_defs.push(format!(
                    "  CONSTRAINT fk_{}_left FOREIGN KEY ({}) REFERENCES {}({})",
                    junction.to_lowercase(),
                    left_fk,
                    rel.left,
                    lpk.name
                ));
                col_defs.push(format!(
                    "  CONSTRAINT fk_{}_right FOREIGN KEY ({}) REFERENCES {}({})",
                    junction.to_lowercase(),
                    right_fk,
                    rel.right,
                    rpk.name
                ));
            }
        }
        out.push_str(&format!(
            "CREATE TABLE {} (\n{}\n);\n",
            junction,
            col_defs.join(",\n")
        ));
    }

    Ok(out)
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
