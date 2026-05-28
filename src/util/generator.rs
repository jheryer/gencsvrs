//! Generator: walks an `ErdAst` and produces one `DataFrame` per entity plus
//! one junction DataFrame per many-to-many relationship.
//!
//! Implements M3 + M4 of `.claude/prds/er-diagram-generator.prd.md`:
//! - Topological order — parents generated before children
//! - FK columns sampled from the parent's PK column per PRD §6.4 cardinality
//! - Implicit FK columns auto-added when no explicit `FK` attribute matches
//!   (PRD §6.5)
//! - M:N relationships emit a junction `<Left>_<Right>` DataFrame with two FK
//!   columns; `}|--|{` enforces ≥1 coverage on both sides

use crate::util::erd_ast::{Cardinality, Entity, ErdAst, Relationship};
use crate::util::fake::create_column;
use crate::util::parser::mermaid_type_to_synthtab;
use crate::util::schema::Schema;
use polars::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenError {
    pub message: String,
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for GenError {}

pub fn generate(
    ast: &ErdAst,
    default_rows: usize,
    rows_per: &HashMap<String, usize>,
) -> Result<Vec<(String, DataFrame)>, GenError> {
    validate_row_counts(ast, default_rows, rows_per)?;

    let order = topological_order(ast)?;
    let mut frames: HashMap<String, DataFrame> = HashMap::new();
    let mut ordered: Vec<(String, DataFrame)> = Vec::new();

    for entity_name in &order {
        let entity = ast
            .entity(entity_name)
            .expect("entity in topo order exists");
        let n = resolve_rows(entity_name, default_rows, rows_per);
        let df = build_entity_frame(entity, ast, &frames, n)?;
        frames.insert(entity_name.clone(), df.clone());
        ordered.push((entity_name.clone(), df));
    }

    for r in &ast.relationships {
        if !r.cardinality.is_many_to_many() {
            continue;
        }
        let junction_name = format!("{}_{}", r.left, r.right);
        let junction_rows = rows_per.get(&junction_name).copied().unwrap_or_else(|| {
            let a = resolve_rows(&r.left, default_rows, rows_per);
            let b = resolve_rows(&r.right, default_rows, rows_per);
            a.max(b)
        });
        let df = build_junction_frame(r, &frames, junction_rows)?;
        ordered.push((junction_name, df));
    }

    Ok(ordered)
}

fn resolve_rows(entity: &str, default_rows: usize, rows_per: &HashMap<String, usize>) -> usize {
    rows_per.get(entity).copied().unwrap_or(default_rows)
}

fn validate_row_counts(
    ast: &ErdAst,
    default_rows: usize,
    rows_per: &HashMap<String, usize>,
) -> Result<(), GenError> {
    for r in &ast.relationships {
        if r.cardinality.requires_equal_counts() {
            let a = resolve_rows(&r.left, default_rows, rows_per);
            let b = resolve_rows(&r.right, default_rows, rows_per);
            if a != b {
                return Err(GenError {
                    message: format!(
                        "entity '{}' set to {b} rows but relationship '{} {:?} {}' requires count({}) == count({}) == {a}",
                        r.right, r.left, r.cardinality, r.right, r.left, r.right
                    ),
                });
            }
        }
    }
    Ok(())
}

fn topological_order(ast: &ErdAst) -> Result<Vec<String>, GenError> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut indeg: HashMap<&str, usize> = HashMap::new();
    for e in &ast.entities {
        graph.insert(e.name.as_str(), Vec::new());
        indeg.insert(e.name.as_str(), 0);
    }
    for r in &ast.relationships {
        if let Some((parent, child)) = r.cardinality.parent_child(&r.left, &r.right) {
            graph.get_mut(parent).unwrap().push(child);
            *indeg.get_mut(child).unwrap() += 1;
        }
    }
    let mut ready: Vec<&str> = indeg
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&k, _)| k)
        .collect();
    ready.sort();
    let mut out: Vec<String> = Vec::new();
    while let Some(node) = ready.pop() {
        out.push(node.to_string());
        if let Some(neighbors) = graph.get(node) {
            let mut newly_ready: Vec<&str> = Vec::new();
            for &n in neighbors {
                let d = indeg.get_mut(n).unwrap();
                *d -= 1;
                if *d == 0 {
                    newly_ready.push(n);
                }
            }
            newly_ready.sort();
            ready.extend(newly_ready);
        }
    }
    if out.len() != ast.entities.len() {
        return Err(GenError {
            message: "cycle detected during topological sort (should have been caught by parser)"
                .to_string(),
        });
    }
    Ok(out)
}

fn fk_targets_for(entity: &str, ast: &ErdAst) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for r in &ast.relationships {
        if r.cardinality.is_many_to_many() {
            continue;
        }
        if let Some((parent, child)) = r.cardinality.parent_child(&r.left, &r.right) {
            if child == entity {
                let fk_name = format!("{}_id", parent.to_lowercase());
                out.push((parent.to_string(), fk_name));
            }
        }
    }
    out
}

fn build_entity_frame(
    entity: &Entity,
    ast: &ErdAst,
    parents: &HashMap<String, DataFrame>,
    n: usize,
) -> Result<DataFrame, GenError> {
    let fk_targets = fk_targets_for(&entity.name, ast);
    let fk_names: HashSet<&str> = fk_targets.iter().map(|(_, n)| n.as_str()).collect();

    let mut columns: Vec<Series> = Vec::new();

    for attr in &entity.attributes {
        if fk_names.contains(attr.name.as_str()) {
            let parent_name = fk_targets
                .iter()
                .find(|(_, n)| n == &attr.name)
                .map(|(p, _)| p.clone())
                .expect("fk_name appeared in set above");
            let series = sample_fk_column(
                &attr.name,
                &parent_name,
                parents,
                n,
                requires_coverage(&entity.name, &parent_name, ast),
            )?;
            columns.push(series);
            continue;
        }

        let synthtab_type = mermaid_type_to_synthtab(&attr.data_type).ok_or_else(|| GenError {
            message: format!(
                "entity '{}': attribute '{}' has unknown type '{}' (parser should have caught this)",
                entity.name, attr.name, attr.data_type
            ),
        })?;

        let schema = Schema {
            name: attr.name.clone(),
            datatype: synthtab_type.to_string(),
            modifier: None,
        };
        let col = create_column(schema, n);
        columns.push(col);
    }

    let attr_names: HashSet<&str> = entity.attributes.iter().map(|a| a.name.as_str()).collect();
    for (parent_name, fk_name) in &fk_targets {
        if attr_names.contains(fk_name.as_str()) {
            continue;
        }
        let series = sample_fk_column(
            fk_name,
            parent_name,
            parents,
            n,
            requires_coverage(&entity.name, parent_name, ast),
        )?;
        columns.push(series);
    }

    DataFrame::new(columns).map_err(|e| GenError {
        message: format!(
            "entity '{}': failed to assemble DataFrame: {e}",
            entity.name
        ),
    })
}

fn requires_coverage(child: &str, parent: &str, ast: &ErdAst) -> bool {
    for r in &ast.relationships {
        if matches!(r.cardinality, Cardinality::OneToMandatoryMany)
            && r.cardinality.parent_child(&r.left, &r.right) == Some((parent, child))
        {
            return true;
        }
    }
    false
}

fn sample_fk_column(
    column_name: &str,
    parent_name: &str,
    parents: &HashMap<String, DataFrame>,
    n: usize,
    require_full_coverage: bool,
) -> Result<Series, GenError> {
    let parent_df = parents.get(parent_name).ok_or_else(|| GenError {
        message: format!(
            "FK column '{column_name}': parent '{parent_name}' not yet generated (topological order bug)"
        ),
    })?;
    let parent_pk_series = parent_df.get_columns().first().ok_or_else(|| GenError {
        message: format!("parent '{parent_name}' has no columns"),
    })?;

    sample_series_with_replacement(column_name, parent_pk_series, n, require_full_coverage)
}

fn sample_series_with_replacement(
    column_name: &str,
    source: &Series,
    n: usize,
    require_full_coverage: bool,
) -> Result<Series, GenError> {
    let parent_len = source.len();
    if parent_len == 0 {
        return Err(GenError {
            message: format!("cannot sample FK '{column_name}' from empty parent column"),
        });
    }

    let mut rng = thread_rng();
    let mut indices: Vec<usize> = (0..n).map(|_| rng.gen_range(0..parent_len)).collect();

    if require_full_coverage && n >= parent_len {
        let mut perm: Vec<usize> = (0..parent_len).collect();
        perm.shuffle(&mut rng);
        for (i, &p) in perm.iter().enumerate() {
            indices[i] = p;
        }
        indices.shuffle(&mut rng);
    }

    take_by_indices(column_name, source, &indices)
}

fn take_by_indices(
    column_name: &str,
    source: &Series,
    indices: &[usize],
) -> Result<Series, GenError> {
    match source.dtype() {
        DataType::Int32 => {
            let ca = source.i32().map_err(|e| GenError {
                message: format!("FK source not Int32: {e}"),
            })?;
            let values: Vec<i32> = indices.iter().map(|&i| ca.get(i).unwrap_or(0)).collect();
            Ok(Series::new(column_name, values))
        }
        DataType::Int64 => {
            let ca = source.i64().map_err(|e| GenError {
                message: format!("FK source not Int64: {e}"),
            })?;
            let values: Vec<i64> = indices.iter().map(|&i| ca.get(i).unwrap_or(0)).collect();
            Ok(Series::new(column_name, values))
        }
        DataType::String => {
            let ca = source.str().map_err(|e| GenError {
                message: format!("FK source not String: {e}"),
            })?;
            let values: Vec<String> = indices
                .iter()
                .map(|&i| ca.get(i).unwrap_or("").to_string())
                .collect();
            Ok(Series::new(column_name, values))
        }
        other => Err(GenError {
            message: format!(
                "FK column '{column_name}': unsupported parent PK dtype {other:?}. Use 'int' or 'uuid'."
            ),
        }),
    }
}

fn build_junction_frame(
    r: &Relationship,
    parents: &HashMap<String, DataFrame>,
    n: usize,
) -> Result<DataFrame, GenError> {
    let left_fk = format!("{}_id", r.left.to_lowercase());
    let right_fk = format!("{}_id", r.right.to_lowercase());

    let left_series = sample_fk_column(
        &left_fk,
        &r.left,
        parents,
        n,
        matches!(r.cardinality, Cardinality::MandatoryManyToMany),
    )?;
    let right_series = sample_fk_column(
        &right_fk,
        &r.right,
        parents,
        n,
        matches!(r.cardinality, Cardinality::MandatoryManyToMany),
    )?;

    DataFrame::new(vec![left_series, right_series]).map_err(|e| GenError {
        message: format!(
            "junction {}_{}: failed to assemble DataFrame: {e}",
            r.left, r.right
        ),
    })
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;
    use crate::util::parser::parse;
    use crate::util::scanner::scan;

    fn ast_from(src: &str) -> ErdAst {
        let toks = scan(src).unwrap();
        parse(toks).unwrap()
    }

    #[test]
    fn generates_topologically_ordered_frames() {
        let src = "\
erDiagram
  CAR { int id PK }
  PERSON { int id PK }
  PERSON ||--o{ CAR : owns
";
        let ast = ast_from(src);
        let frames = generate(&ast, 5, &HashMap::new()).unwrap();
        let names: Vec<&str> = frames.iter().map(|(n, _)| n.as_str()).collect();
        let person_idx = names.iter().position(|&n| n == "PERSON").unwrap();
        let car_idx = names.iter().position(|&n| n == "CAR").unwrap();
        assert!(
            person_idx < car_idx,
            "PERSON must come before CAR, got {names:?}"
        );
    }

    #[test]
    fn one_to_many_fk_values_exist_in_parent_pk() {
        let src = "\
erDiagram
  PARENT { int id PK }
  CHILD { int id PK }
  PARENT ||--o{ CHILD : has
";
        let ast = ast_from(src);
        let frames = generate(&ast, 10, &HashMap::new()).unwrap();
        let parent = frames.iter().find(|(n, _)| n == "PARENT").unwrap();
        let child = frames.iter().find(|(n, _)| n == "CHILD").unwrap();

        let parent_ids: HashSet<i32> = parent
            .1
            .column("id")
            .unwrap()
            .i32()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();
        let child_fks: Vec<i32> = child
            .1
            .column("parent_id")
            .unwrap()
            .i32()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();

        assert_eq!(child_fks.len(), 10);
        for fk in &child_fks {
            assert!(
                parent_ids.contains(fk),
                "child fk {fk} not in parent ids {parent_ids:?}"
            );
        }
    }

    #[test]
    fn many_to_many_emits_junction_table() {
        let src = "\
erDiagram
  STUDENT { int id PK }
  COURSE { int id PK }
  STUDENT }o--o{ COURSE : enrolled
";
        let ast = ast_from(src);
        let frames = generate(&ast, 5, &HashMap::new()).unwrap();
        let names: Vec<&str> = frames.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"STUDENT_COURSE"), "got: {names:?}");
        let junction = frames.iter().find(|(n, _)| n == "STUDENT_COURSE").unwrap();
        let cols: Vec<&str> = junction.1.get_column_names();
        assert_eq!(cols, vec!["student_id", "course_id"]);
    }

    #[test]
    fn mandatory_one_to_many_covers_every_parent() {
        let src = "\
erDiagram
  PARENT { int id PK }
  CHILD { int id PK }
  PARENT ||--|{ CHILD : has
";
        let ast = ast_from(src);
        let mut rows = HashMap::new();
        rows.insert("PARENT".to_string(), 4);
        rows.insert("CHILD".to_string(), 12);
        let frames = generate(&ast, 10, &rows).unwrap();
        let child = frames.iter().find(|(n, _)| n == "CHILD").unwrap();
        let child_fks: HashSet<i32> = child
            .1
            .column("parent_id")
            .unwrap()
            .i32()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(
            child_fks.len(),
            4,
            "expected all 4 parents covered, got {child_fks:?}"
        );
    }

    #[test]
    fn one_to_one_requires_equal_counts() {
        let src = "\
erDiagram
  USER { int id PK }
  PROFILE { int id PK }
  USER ||--|| PROFILE : has
";
        let ast = ast_from(src);
        let mut rows = HashMap::new();
        rows.insert("USER".to_string(), 5);
        rows.insert("PROFILE".to_string(), 10);
        let err = generate(&ast, 5, &rows).unwrap_err();
        assert!(
            err.message.contains("requires count"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn implicit_fk_column_is_added_when_no_attribute_matches() {
        let src = "\
erDiagram
  AUTHOR { int id PK }
  BOOK { int id PK }
  AUTHOR ||--o{ BOOK : writes
";
        let ast = ast_from(src);
        let frames = generate(&ast, 5, &HashMap::new()).unwrap();
        let book = frames.iter().find(|(n, _)| n == "BOOK").unwrap();
        let cols: Vec<&str> = book.1.get_column_names();
        assert!(cols.contains(&"author_id"), "got cols: {cols:?}");
    }

    #[test]
    fn explicit_fk_attribute_is_used_not_duplicated() {
        let src = "\
erDiagram
  AUTHOR { int id PK }
  BOOK { int id PK
         int author_id FK }
  AUTHOR ||--o{ BOOK : writes
";
        let ast = ast_from(src);
        let frames = generate(&ast, 5, &HashMap::new()).unwrap();
        let book = frames.iter().find(|(n, _)| n == "BOOK").unwrap();
        let cols: Vec<&str> = book.1.get_column_names();
        let count = cols.iter().filter(|&&c| c == "author_id").count();
        assert_eq!(
            count, 1,
            "expected author_id exactly once, got cols: {cols:?}"
        );
    }
}
