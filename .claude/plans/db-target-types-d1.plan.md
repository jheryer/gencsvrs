# Plan: Database-Target Types — D1 (`Dialect` Enum + Type-Mapping Table)

**Source PRD**: `.claude/prds/db-target-types.prd.md`
**Selected Milestone**: D1 — `Dialect` enum + complete type-mapping table + unit tests (no I/O)
**Target branch**: `feature/db-targets-d1` (cut from `main`; can land independently of `feature/erd-v2`)
**Complexity**: Small

## Summary

Land the foundational `Dialect` enum and the full `gencsv-type × dialect → SQL-type-string` mapping table from PRD §6.2 as a pure-library module with no I/O, no CLI changes, no `Output` sink. This milestone is a lookup table behind a small API; everything else (DDL emission, load commands, Parquet logical types, ER integration) is layered on top of it in D2–D5. Choosing to land this independently of the ER work is deliberate — D1 has zero behavioural overlap with ER and no shared files, so it can ship in parallel.

## Patterns to Mirror

| Category | Source | Pattern |
|---|---|---|
| Sub-module re-export | `src/util/mod.rs` | Add `pub mod dialect;` alongside `schema`, `fake`, `dataframe`, `output` |
| Error type | `src/lib.rs:7` | `Box<dyn Error>` via `format!("...: {e}")` context — repo convention, NOT `thiserror` (see CLAUDE.md "Error handling") |
| Inline test module (no `#[cfg(test)]`) | `src/util/schema.rs:58-93`, `src/util/fake.rs:270-…` | `mod test { #![allow(unused_imports)] use super::*; ... }` per CLAUDE.md convention |
| Match-based dispatch over type strings | `src/util/fake.rs:60-90` `create_column` | `match element.datatype.as_str() { "INT" => …, "STRING" => …, _ => fall-through }` — keep the same shape so a future reader can pattern-match between the two tables |
| Enum + `as_str()` / `from_str` pair | None in this repo yet — clean greenfield | Use `#[derive(clap::ValueEnum)]` so D2/CLI work in a later milestone can attach `--target` flag with zero refactor |
| String literals over `&'static str` constants | `src/util/fake.rs:60` (type strings inline) | Keep mapping table values inline; don't pre-allocate `const POSTGRES_INT: &str = "INTEGER"` constants until something needs them by name |

## Requirements Restatement

Build a single new file `src/util/dialect.rs` that exposes:

```rust
pub enum Dialect { Mysql, Postgres, Sqlserver, Bigquery, Spark }

impl Dialect {
    pub fn as_str(&self) -> &'static str;   // "mysql" | "postgres" | ...
    pub fn from_str(s: &str) -> Result<Self, DialectError>;
}

pub struct DialectError { pub message: String }   // Display + Error

pub fn to_sql_type(gencsv_type: &str, dialect: Dialect, is_pk: bool) -> Result<String, DialectError>;
```

All 24 rows in PRD §6.2 must be covered. `is_pk` only changes the output for `INT_INC` (per the PRD table: PK gets `INT AUTO_INCREMENT PRIMARY KEY` / `SERIAL PRIMARY KEY` / etc., non-PK gets `INT NOT NULL` / `INTEGER NOT NULL` / etc.). For all other types, `is_pk` is accepted but ignored — we don't decorate non-`INT_INC` PKs in D1.

Unknown `gencsv_type` returns `Err(DialectError { message: "type 'foo' has no '{dialect}' mapping; supported mappings: see docs/DIALECTS.md" })` per PRD §7. The `docs/DIALECTS.md` reference is forward-looking — file lands in D6, but the error string must be present from D1 so we don't need to update it later.

**Explicitly NOT in D1**: no `Output` trait impl, no file emission, no CLI flag, no DDL string assembly (`CREATE TABLE ...`), no load-command rendering, no Parquet logical-type wiring, no schema.rs changes. Those are D2–D5.

## Files to Change

| File | Action | Why |
|---|---|---|
| `src/util/dialect.rs` | CREATE | The whole milestone — `Dialect` enum + mapping function + inline unit tests |
| `src/util/mod.rs` | UPDATE | `pub mod dialect;` |
| `Cargo.toml` | UPDATE (maybe) | Only if `clap::ValueEnum` derive needs the `derive` feature — already enabled (`clap = { version = "4.5", features = ["derive"] }` per CLAUDE.md). Likely no change. |

No changes to `src/main.rs`, `src/lib.rs`, `src/util/schema.rs`, `src/util/fake.rs`, `src/util/dataframe.rs`, `src/util/output.rs`, `tests/cli.rs`. D1 is purely additive at the library level.

## Tasks (TDD order — RED → GREEN → REFACTOR)

### Task 1: Branch
- **Action**: `git checkout -b feature/db-targets-d1` from `main`.
- **Validate**: `git status` clean; on new branch.

### Task 2: Module skeleton + `Dialect` enum + failing tests
- **Action**: Create `src/util/dialect.rs` with:
  - `#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)] pub enum Dialect { Mysql, Postgres, Sqlserver, Bigquery, Spark }`
  - `pub struct DialectError { pub message: String }` with `Display` + `Error` impls
  - `impl Dialect { pub fn as_str(&self) -> &'static str { ... } pub fn from_str(s: &str) -> Result<Self, DialectError> { ... } }` — body for `as_str` is the only thing implemented; `from_str` and `to_sql_type` return `Err` stubs.
  - `pub fn to_sql_type(...) -> Result<String, DialectError> { Err(DialectError { message: "not implemented".into() }) }`
  - Add `pub mod dialect;` to `src/util/mod.rs`.
  - Write inline tests (`mod test { #![allow(unused_imports)] use super::*; ... }`) — see Task 3 for the full test matrix. Tests must fail.
- **Validate**: `cargo build` succeeds; `cargo test dialect` reports failures (not panics).

### Task 3: Implement `from_str` + full mapping table
- **Action**: Drive the implementation off the failing tests. Group tests by dialect — one test function per `(dialect, type-class)` clump rather than 24×5 individual tests, to keep the file readable:

  | Test name | Asserts |
  |---|---|
  | `from_str_round_trips_all_dialects` | `Dialect::from_str("postgres").unwrap().as_str() == "postgres"` for all 5 dialects |
  | `from_str_rejects_unknown_target` | `Dialect::from_str("redshift").is_err()`; error message contains `"unsupported target 'redshift'"` and lists the 5 supported names (matches PRD §7) |
  | `from_str_is_case_insensitive` | `Dialect::from_str("MySQL")` and `Dialect::from_str("MYSQL")` both succeed (defensive — `--target` CLI input is user-typed) |
  | `int_inc_pk_maps_per_dialect` | `to_sql_type("INT_INC", Mysql, true) == "INT AUTO_INCREMENT PRIMARY KEY"`, plus Postgres → `"SERIAL PRIMARY KEY"`, SQL Server → `"INT IDENTITY(1,1) PRIMARY KEY"`, BigQuery → contains `INT64`, Spark → contains `INT` |
  | `int_inc_non_pk_drops_identity` | `to_sql_type("INT_INC", Mysql, false) == "INT NOT NULL"`, etc. |
  | `numeric_types_map_per_dialect` | Covers `INT`, `INT_RNG`, `DECIMAL`, `PRICE`, `LAT`, `LON` across all 5 dialects |
  | `string_types_map_per_dialect` | Covers `STRING`, `VALUE`, `DIGIT`, `NAME`, `FIRST_NAME`, `LAST_NAME`, `SSN`, `ZIP_CODE`, `COUNTRY_CODE`, `STATE_NAME`, `STATE_ABBR`, `PHONE`, `LOREM_WORD`, `LOREM_TITLE`, `LOREM_SENTENCE`, `LOREM_PARAGRAPH`, `UUID` |
  | `date_time_types_map_per_dialect` | Covers `DATE`, `TIME`, `DATE_TIME` — including Spark's `TIME → STRING` exception |
  | `unknown_type_returns_error` | `to_sql_type("FOO", Postgres, false)` returns error matching `"type 'FOO' has no 'postgres' mapping"` |
  | `is_pk_ignored_for_non_int_inc` | `to_sql_type("STRING", Postgres, true) == to_sql_type("STRING", Postgres, false)` — explicit invariant assertion |

- Implementation shape:
  ```rust
  pub fn to_sql_type(gencsv_type: &str, dialect: Dialect, is_pk: bool) -> Result<String, DialectError> {
      let mapped = match (dialect, gencsv_type) {
          (Dialect::Mysql,    "INT_INC") if is_pk => "INT AUTO_INCREMENT PRIMARY KEY",
          (Dialect::Mysql,    "INT_INC")           => "INT NOT NULL",
          (Dialect::Postgres, "INT_INC") if is_pk => "SERIAL PRIMARY KEY",
          (Dialect::Postgres, "INT_INC")           => "INTEGER NOT NULL",
          // ... rest of the matrix
          (_, unknown) => return Err(DialectError {
              message: format!("type '{unknown}' has no '{}' mapping; supported mappings: see docs/DIALECTS.md", dialect.as_str()),
          }),
      };
      Ok(mapped.to_string())
  }
  ```
  Tuple-match `(Dialect, &str)` keeps the table flat and greppable — readers can `rg "INT_INC"` and see every dialect's row at once. Resist the urge to introduce a `HashMap<(Dialect, String), String>` — that adds runtime allocation for no readability gain at 24×5 entries.

- **Mirror**: `src/util/fake.rs:60-90` (`match element.datatype.as_str()` style).
- **Validate**: `cargo test dialect` all green; `cargo clippy --all-targets -- -D warnings` clean.

### Task 4: Edge-case + invariant tests
- **Action**: Add three more inline tests that lock down behaviour the PRD implies but doesn't spell out:
  - `every_type_in_readme_has_at_least_one_dialect_mapping` — table-driven test that iterates over a hardcoded `const SUPPORTED_TYPES: &[&str] = &["INT_INC", "INT", ...]` (the 24 rows from PRD §6.2) and asserts `to_sql_type(t, d, false).is_ok()` for every `(t, d)` combo. This is the contract: D1 is "complete table coverage", and this single test enforces it.
  - `error_messages_quote_the_user_input_verbatim` — `to_sql_type("int", ...)` (lowercase) returns error mentioning `'int'`, not `'INT'`. Prevents silent normalisation that would hide typos.
  - `dialect_display_is_lowercase` — `Dialect::Mysql.as_str() == "mysql"` etc.; matches `--target mysql` CLI convention from PRD §5.1.
- **Validate**: `cargo test dialect` — all green.

### Task 5: Quality gates + commit
- **Action**: Run the full pre-commit suite. Conventional commit: `feat: add Dialect enum and gencsv-type -> SQL-type mapping (D1)`.
- **Validate**: see Validation block below.

## Validation

```bash
# Formatting
cargo fmt --all -- --check

# Lints
cargo clippy --all-targets -- -D warnings

# Unit tests (no integration tests added in D1)
cargo test dialect

# Full suite — confirm no regressions elsewhere
cargo test

# Manual sanity: dialect module exposes the surface D2 will consume
cargo doc --no-deps --open    # optional; verify pub items are documented
```

Coverage gate (`cargo llvm-cov --fail-under-lines 80`) is a D6 acceptance criterion in the PRD, not D1 — CLAUDE.md notes the repo has no coverage tooling wired up yet. Skip in D1.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Mapping table row drift between PRD §6.2 and `to_sql_type` implementation | **High** | Single source of truth in `dialect.rs`. When D6 ships `docs/DIALECTS.md`, the doc gets generated from a table-driven test or a const slice — not maintained by hand. For D1, the `every_type_in_readme_has_at_least_one_dialect_mapping` test guarantees coverage. |
| Type-string mismatch between `gencsv` schema and `to_sql_type` keys (e.g. `"INT"` vs `"int"`) | High | `create_column` in `src/util/fake.rs:60` matches uppercase. `to_sql_type` must take the same uppercase key. Add a doc comment on `to_sql_type` pinning this contract: `/// `gencsv_type` must be one of the uppercase keys accepted by `create_column`.` |
| `clap::ValueEnum` derive adds a build dependency we didn't intend | Low | Clap derive is already on per CLAUDE.md (`clap = "4.5 derive"`). No new crate. |
| `Dialect::from_str` collides with `std::str::FromStr` trait impl readers might expect | Low | Implement as inherent method `from_str(s: &str) -> Result<Self, DialectError>` rather than the `FromStr` trait — keeps error type local, avoids `std::str::FromStr::Err` boilerplate. Document in the method's doc-comment. |
| Future DDL emitter (D2) needs more context than `(type, dialect, is_pk)` (e.g. nullability, FK target, length override) | Medium | D1 keeps the signature minimal. D2 will likely wrap `to_sql_type` rather than change it — pass a `ColumnSpec` struct in D2 if needed. Don't pre-design that struct in D1. |
| Mapping table makes `dialect.rs` >800 lines | Low | 24 types × 5 dialects = 120 match arms — comfortably under the 800-line target from global rules. If it bloats, split into one `fn map_<dialect>(t, is_pk)` per dialect and dispatch from `to_sql_type`. |

## Acceptance

- [ ] `feature/db-targets-d1` branch cut fresh from `main`
- [ ] `src/util/dialect.rs` exists with `pub enum Dialect`, `pub fn to_sql_type`, `pub struct DialectError`
- [ ] All 24 rows of PRD §6.2 mapping table are covered (enforced by `every_type_in_readme_has_at_least_one_dialect_mapping` test)
- [ ] `Dialect::from_str("redshift")` returns the exact error string from PRD §7
- [ ] `to_sql_type("FOO", ...)` returns the exact error string from PRD §7 (with `docs/DIALECTS.md` reference, even though that doc lands in D6)
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` all green
- [ ] No new crate added to `Cargo.toml`
- [ ] Existing flat-table mode behaviour is unchanged (no files touched outside `src/util/dialect.rs` + `src/util/mod.rs`)
- [ ] D1 row in PRD §9 marked `in-progress` with link to this plan

## Out of Scope (deferred to later milestones)

- `--target <dialect>` CLI flag → **D2** (needs `DdlEmitter` to make the flag meaningful)
- `DdlEmitter` (writes `<table>.ddl.<dialect>.sql`) → **D2**
- `LoadCmdEmitter` (writes `<table>.load.<dialect>.{sql,sh,py}`) → **D3**
- Parquet logical-type alignment for BigQuery / Spark → **D4**
- ER-mode integration (per-entity DDL + FK constraints) → **D5** (requires ER PRD M1–M5 complete)
- `docs/DIALECTS.md` + README quickstart → **D6**
- `--no-ddl`, `--no-load`, `--no-data` flags + behaviour matrix → **D2/D3**
- Identifier quoting per dialect (backticks vs double-quotes) → **D2** (only matters at DDL emission)
- Reserved-word collision handling → **D2**

## Sequencing Note

This milestone is **independent of the ER PRD**. D1 lands a pure-library lookup table; nothing in `feature/erd-v2` (ER M1) reads from `src/util/dialect.rs` or vice-versa. The two branches can be developed and merged in either order without conflict. The PRD-level "Sequenced after ER PRD's M1–M5" sequencing constraint **only binds D5**, which is the milestone that actually plugs into the ER pipeline. D1–D4 may be worked in parallel with the ER track.
