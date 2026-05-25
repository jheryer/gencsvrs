# PRD: Database-Target Type Definitions

**Status:** Draft
**Owner:** jheryer
**Created:** 2026-05-25
**Target branch:** `feature/db-targets` (fresh from `main`, after ER PRD lands)
**Depends on:** [`er-diagram-generator.prd.md`](./er-diagram-generator.prd.md) for milestone D5 (ER-mode DDL emission)

---

## 1. Problem

`gencsv` produces CSV (typeless) and Parquet (typed, but generically). Users importing into MySQL, Postgres, SQL Server, BigQuery, or Spark have to:

1. Hand-write a `CREATE TABLE` matching the generated columns
2. Remember the right `LOAD DATA` / `\copy` / `BULK INSERT` / `bq load` / Spark DataFrame incantation
3. Hope that Parquet logical types align with the target database's expectations

This friction defeats the "generate fixture data fast" promise of the tool.

## 2. Goal

When the user passes `--target <dialect>`, `gencsv` emits everything needed to land the data in that database:

1. Dialect-correct **Parquet logical types** (when output format is parquet)
2. A **`CREATE TABLE` DDL file** matching the generated columns
3. An **example load command** for the target dialect
4. All of the above composing cleanly with both flat-table mode (`gencsv -s …`) and ER mode (`gencsv er …`)

## 3. Non-Goals

- Direct DB connection / pushing rows (use the generated load command)
- Per-column DB-type overrides in the schema string (Phase 2)
- Bounded numeric variants (`TINYINT`, `SMALLINT`, `BIGINT` ranges) (Phase 2)
- Schema migrations / `ALTER TABLE` emission
- Dialect-specific generators (e.g. Postgres-flavored JSON column values)
- Composite primary keys (matches ER PRD scope)

## 4. User Stories

| As a… | I want to… | So that… |
|---|---|---|
| Backend engineer | Run `gencsv -s "id:INT_INC,email:STRING" -r 5000 -c -f users.csv --target postgres` and get `users.csv` + `users.ddl.postgres.sql` + `users.load.postgres.sql` | I can `psql -f users.ddl.postgres.sql && psql -f users.load.postgres.sql` and be done |
| Data engineer | Pipe an ER diagram + `--target bigquery` and get one Parquet per entity plus dialect-correct DDL | I can `bq load --source_format=PARQUET` each table directly |
| QA engineer | See FK constraints in the emitted DDL for ER-mode output | Loading the data exercises the same referential integrity my production schema does |
| Anyone | See a clear error if I ask for a target/type combo we don't support | I'm not chasing silent type coercions at load time |

## 5. Functional Requirements

### 5.1 CLI

Existing commands grow three new flags:

```
gencsv [existing flags...]
    [--target mysql|postgres|sqlserver|bigquery|spark]
    [--no-ddl]                  # suppress DDL emission when --target is set
    [--no-load]                 # suppress load-command emission when --target is set
    [--no-data]                 # emit only DDL + load command, skip data files (handy for inspection)
```

Same flags work on `gencsv er <FILE> --target postgres --out DIR`.

### 5.2 Output bundle (one entity, target = `postgres`)

```
users.csv                       # existing data file
users.ddl.postgres.sql          # CREATE TABLE
users.load.postgres.sql         # \copy users FROM 'users.csv' ...
```

For BigQuery and Spark the load file is a code snippet, not SQL:

```
users.load.bigquery.sh          # bq load --source_format=PARQUET ...
users.load.spark.py             # spark.read.parquet(...).write.saveAsTable(...)
```

### 5.3 Behavior matrix

| `--target` set | `--no-ddl` | `--no-load` | `--no-data` | Result |
|---|---|---|---|---|
| no | — | — | — | Current behavior unchanged |
| yes | no | no | no | Data + DDL + load command (default) |
| yes | yes | no | no | Data + load command |
| yes | no | yes | no | Data + DDL |
| yes | yes | yes | no | Data only (no point — warn the user) |
| yes | no | no | yes | DDL + load command, no data file |

### 5.4 Parquet logical types

When `--parquet` and `--target` are both set, the Parquet writer uses dialect-aligned logical types:

- BigQuery target → `INT64`, `NUMERIC`, `TIMESTAMP_MICROS`, `STRING`
- Spark target → `INT32`/`INT64`, `DECIMAL(p,s)`, `TIMESTAMP_MICROS`, `STRING`
- MySQL / Postgres / SQL Server → physical-type defaults (these dialects rely on the DDL-defined column type rather than Parquet logical types during CSV load; Parquet load paths are non-native for these three)

Caveats per dialect documented in `docs/DIALECTS.md`.

---

## 6. Supported Targets and Type Mappings

### 6.1 Targets in scope

| Dialect ID | Display name | Native load formats |
|---|---|---|
| `mysql` | MySQL 8+ | CSV (`LOAD DATA INFILE`) |
| `postgres` | PostgreSQL 14+ | CSV (`\copy`, `COPY`) |
| `sqlserver` | SQL Server 2019+ | CSV (`BULK INSERT`) |
| `bigquery` | Google BigQuery | Parquet (preferred), CSV |
| `spark` | Apache Spark 3+ | Parquet (preferred), CSV |

### 6.2 Type mapping table (canonical reference)

| gencsv type | MySQL | Postgres | SQL Server | BigQuery | Spark |
|---|---|---|---|---|---|
| `INT_INC` (PK) | `INT AUTO_INCREMENT PRIMARY KEY` | `SERIAL PRIMARY KEY` | `INT IDENTITY(1,1) PRIMARY KEY` | `INT64` + comment "// auto-increment not native" | `INT` + comment "// auto-increment not native" |
| `INT_INC` (non-PK) | `INT NOT NULL` | `INTEGER NOT NULL` | `INT NOT NULL` | `INT64` | `INT` |
| `INT` | `INT` | `INTEGER` | `INT` | `INT64` | `INT` |
| `INT_RNG` | `INT` | `INTEGER` | `INT` | `INT64` | `INT` |
| `DIGIT` | `CHAR(1)` | `CHAR(1)` | `CHAR(1)` | `STRING` | `STRING` |
| `DECIMAL` | `DECIMAL(10,2)` | `NUMERIC(10,2)` | `DECIMAL(10,2)` | `NUMERIC` | `DECIMAL(10,2)` |
| `PRICE` | `DECIMAL(10,2)` | `NUMERIC(10,2)` | `MONEY` | `NUMERIC` | `DECIMAL(10,2)` |
| `STRING` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `VALUE` | `VARCHAR(50)` | `VARCHAR(50)` | `VARCHAR(50)` | `STRING` | `STRING` |
| `DATE` | `DATE` | `DATE` | `DATE` | `DATE` | `DATE` |
| `TIME` | `TIME` | `TIME` | `TIME` | `TIME` | `STRING` (Spark has no TIME type) |
| `DATE_TIME` | `DATETIME` | `TIMESTAMP` | `DATETIME2` | `TIMESTAMP` | `TIMESTAMP` |
| `NAME` / `FIRST_NAME` / `LAST_NAME` | `VARCHAR(100)` | `TEXT` | `NVARCHAR(100)` | `STRING` | `STRING` |
| `SSN` | `CHAR(11)` | `CHAR(11)` | `CHAR(11)` | `STRING` | `STRING` |
| `ZIP_CODE` | `VARCHAR(10)` | `VARCHAR(10)` | `VARCHAR(10)` | `STRING` | `STRING` |
| `COUNTRY_CODE` | `CHAR(2)` | `CHAR(2)` | `CHAR(2)` | `STRING` | `STRING` |
| `STATE_NAME` | `VARCHAR(64)` | `VARCHAR(64)` | `NVARCHAR(64)` | `STRING` | `STRING` |
| `STATE_ABBR` | `CHAR(2)` | `CHAR(2)` | `CHAR(2)` | `STRING` | `STRING` |
| `LAT` / `LON` | `DECIMAL(9,6)` | `NUMERIC(9,6)` | `DECIMAL(9,6)` | `NUMERIC` | `DECIMAL(9,6)` |
| `PHONE` | `VARCHAR(20)` | `VARCHAR(20)` | `VARCHAR(20)` | `STRING` | `STRING` |
| `LOREM_WORD` | `VARCHAR(50)` | `VARCHAR(50)` | `NVARCHAR(50)` | `STRING` | `STRING` |
| `LOREM_TITLE` | `VARCHAR(200)` | `VARCHAR(200)` | `NVARCHAR(200)` | `STRING` | `STRING` |
| `LOREM_SENTENCE` | `TEXT` | `TEXT` | `NVARCHAR(MAX)` | `STRING` | `STRING` |
| `LOREM_PARAGRAPH` | `TEXT` | `TEXT` | `NVARCHAR(MAX)` | `STRING` | `STRING` |
| `UUID` | `CHAR(36)` | `UUID` | `UNIQUEIDENTIFIER` | `STRING` | `STRING` |

### 6.3 Load command templates

| Target | Load template |
|---|---|
| `mysql` | `LOAD DATA LOCAL INFILE '{file}' INTO TABLE {table} FIELDS TERMINATED BY ',' ENCLOSED BY '"' LINES TERMINATED BY '\n' IGNORE 1 ROWS;` |
| `postgres` | `\copy {table} FROM '{file}' WITH (FORMAT csv, HEADER true);` |
| `sqlserver` | `BULK INSERT {table} FROM '{file}' WITH (FORMAT = 'CSV', FIRSTROW = 2, KEEPIDENTITY);` |
| `bigquery` (CSV) | `bq load --source_format=CSV --skip_leading_rows=1 {dataset}.{table} {file}` |
| `bigquery` (Parquet) | `bq load --source_format=PARQUET {dataset}.{table} {file}` |
| `spark` (CSV) | `spark.read.option("header", True).csv("{file}").write.saveAsTable("{table}")` |
| `spark` (Parquet) | `spark.read.parquet("{file}").write.saveAsTable("{table}")` |

Template placeholders `{file}`, `{table}`, `{dataset}` are filled at emit time. `{dataset}` for BigQuery defaults to `dataset` with a generated comment telling the user to substitute their own.

### 6.4 ER-mode DDL emission (D5)

For ER input, DDL emission additionally produces:

1. One `CREATE TABLE` per entity (including the auto-generated junction tables)
2. `FOREIGN KEY (<fk_col>) REFERENCES <parent>(<parent_pk>)` constraints
3. Tables emitted in topological order so the DDL file is replayable top-to-bottom

---

## 7. Rejected Inputs (with required error messages)

| Input | Why rejected | Required error |
|---|---|---|
| `--target redshift` (or any unsupported dialect) | Out of scope | `unsupported target 'redshift'; supported: mysql, postgres, sqlserver, bigquery, spark` |
| `--no-data --no-ddl --no-load` together | Nothing to do | `--no-data --no-ddl --no-load: nothing to emit` |
| `--target X` without `--file-target` | Need a path to write companion files | `--target requires --file-target so DDL and load files can be placed next to data` |
| Type with no dialect mapping (future custom types) | Mapping table miss | `type 'foo' has no '{dialect}' mapping; supported mappings: see docs/DIALECTS.md` |
| `--target spark` + `--csv` + relying on auto-load | Spark loads better from Parquet | Warning (not error): `target spark prefers Parquet; consider --parquet` |

All errors exit non-zero and produce no partial output.

---

## 8. Architecture

```
                          ┌──────────────────────────────────┐
                          │ Args: --target --no-ddl ...      │
                          └────────────┬─────────────────────┘
                                       v
┌────────────────┐    ┌────────────────────────────────┐
│ Schema parser  │ -> │ TypeResolver (gencsv types)    │
└────────────────┘    └────────────────┬───────────────┘
                                       │
                          ┌────────────┴────────────┐
                          v                         v
                 ┌──────────────────┐    ┌──────────────────────┐
                 │ DataFrame build  │    │ Dialect mapper       │
                 │ (existing)       │    │ src/util/dialect.rs  │
                 └────────┬─────────┘    └──────────┬───────────┘
                          │                         │
                          v                         v
                 ┌────────────────┐      ┌──────────────────────┐
                 │ MultiSink      │      │ DdlEmitter           │
                 │ + ParquetType  │ <-── │ LoadCmdEmitter       │
                 │ overrides      │      │ src/util/ddl.rs      │
                 └────────────────┘      │ src/util/load_cmd.rs │
                          │              └──────────────────────┘
                          v                         │
                  data.{csv,parquet}                v
                                          data.ddl.<dialect>.sql
                                          data.load.<dialect>.{sql,sh,py}
```

### Files to create / change

| File | Action | Why |
|---|---|---|
| `src/util/dialect.rs` | CREATE | `Dialect` enum + `to_sql_type(gencsv_type, is_pk) -> &str` per dialect; load template lookup |
| `src/util/ddl.rs` | CREATE | `emit_create_table(table_name, columns, dialect) -> String`; FK constraint support |
| `src/util/load_cmd.rs` | CREATE | Template renderer per dialect; choose file extension |
| `src/util/output.rs` | UPDATE | Add `DdlFile`, `LoadCmdFile` sinks; teach `ParquetFile::new_for_dialect` |
| `src/util/schema.rs` | UPDATE | Add optional `is_pk: bool` on `Schema` (defaults to false; back-compat) |
| `src/main.rs` | UPDATE | New `--target`, `--no-ddl`, `--no-load`, `--no-data` clap args |
| `src/lib.rs` | UPDATE | Route `--target` through `run()`; orchestrate companion-file emission |
| `tests/fixtures/dialects/*.expected.sql` | CREATE | Golden DDL per (table × dialect) |
| `tests/cli.rs` | UPDATE | One round-trip integration test per dialect |
| `docs/DIALECTS.md` | CREATE | Full type-mapping reference + per-dialect caveats |
| `README.md` | UPDATE | Quickstart with `--target postgres` example |

### Patterns to mirror

| Category | Source | Pattern |
|---|---|---|
| Errors | `src/util/dataframe.rs:21` | `Box<dyn Error>` with `format!("...: {e}")` context |
| Sink trait | `src/util/output.rs:7` | New emitters implement `Output` so they slot into `run()` uniformly |
| Type dispatch | `src/util/fake.rs:60` | `match dialect { Mysql => ... }` per type, no dyn dispatch |
| Tests | `src/util/fake.rs:270` | Co-located `#[cfg(test)] mod test` with golden-string assertions |
| Integration | `tests/cli.rs` | `assert_cmd` + file existence + content snapshot |

---

## 9. Delivery Milestones

| # | Milestone | Plan | Status |
|---|---|---|---|
| D1 | `Dialect` enum + complete type-mapping table + unit tests (no I/O) | [.claude/plans/db-target-types-d1.plan.md](../plans/db-target-types-d1.plan.md) | in-progress |
| D2 | `DdlEmitter` writes `<table>.ddl.<dialect>.sql` for flat-table mode + per-dialect golden tests | _pending_ | pending |
| D3 | `LoadCmdEmitter` writes `<table>.load.<dialect>.{sql,sh,py}` + per-dialect golden tests | _pending_ | pending |
| D4 | Parquet logical-type alignment per dialect (BigQuery, Spark) | _pending_ | pending |
| D5 | ER-mode integration: emit one DDL per entity + FK constraints; junction tables get DDL too (depends on ER PRD M5) | _pending_ | pending |
| D6 | `docs/DIALECTS.md` + README update + ≥80% coverage maintained | _pending_ | pending |

Each milestone follows the TDD workflow (test-first, RED→GREEN→refactor) and lands as its own PR. **Sequenced after the ER PRD's M1–M5 complete** so D5 has a real ER pipeline to plug into.

---

## 10. Acceptance Criteria

- [ ] `gencsv -s "id:INT_INC,email:STRING" -r 100 -c -f users.csv --target postgres` produces `users.csv`, `users.ddl.postgres.sql`, `users.load.postgres.sql`
- [ ] Loading the generated bundle into the matching DB succeeds end-to-end for at least Postgres and MySQL (manual smoke test recorded in `docs/DIALECTS.md`)
- [ ] All five dialects produce DDL that matches `tests/fixtures/dialects/*.expected.sql`
- [ ] ER-mode + `--target` emits per-entity DDL with FK constraints in topological replay order
- [ ] All errors in §7 are reproducible via fixture inputs and produce documented messages
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo llvm-cov --fail-under-lines 80` pass in CI
- [ ] No regressions in flat-table mode without `--target` (existing tests stay green)

---

## 11. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Generated value ranges exceed target type (`fake_int()` is `i32::MAX`, doesn't fit MySQL `SMALLINT`) | High | Document safe ranges per gencsv type; defer bounded variants to Phase 2 |
| Polars 0.38 Parquet writer doesn't expose all logical-type knobs | Medium | Use physical-type defaults where logical types aren't reachable; document gaps |
| SQL Server `IDENTITY` columns reject inserts without `KEEPIDENTITY` | High | Emit `WITH (KEEPIDENTITY)` in the BULK INSERT template; document in DIALECTS.md |
| BigQuery `bq` CLI requires `{dataset}` placeholder the user must fill in | Medium | Leave as `dataset` with `# TODO: replace` comment; document in DIALECTS.md |
| Spark "load command" is really Scala/Python, not shell | Low | Choose file extension by dialect (`.sh` for MySQL/Postgres/SQLServer/BigQuery, `.py` for Spark) |
| Combinatorial test explosion (5 dialects × ~25 types × CSV/Parquet) | Medium | Golden-file tests cover type mapping; one integration round-trip per dialect; trust unit tests for the matrix |
| Composite FKs differ across dialects | Low | Out of scope in v1 (matches ER PRD); single-column FK syntax is portable |
| Reserved-word collisions in DDL (e.g. entity named `ORDER`) | Medium | Wrap identifiers in backticks (MySQL), double quotes (Postgres, SQL Server, BigQuery), backticks (Spark) — per-dialect quoter |

---

## 12. Open Questions

- Should the load command file be executable (`chmod +x`) for shell targets? (Default: no — user opts in.)
- For BigQuery, should we emit a `bq mk` command for the dataset too? (Default: no — out of scope; document the prerequisite.)
- Should `--target` accept a comma list (`--target mysql,postgres`) to emit bundles for multiple dialects in one run? (Default: no in v1 — single target per invocation; revisit if users ask.)
- Should `STRING` width default to `255` (current proposal) or `TEXT` everywhere? (Default per table; revisit after first user feedback.)

---

## 13. References

- ER PRD (sibling): [`er-diagram-generator.prd.md`](./er-diagram-generator.prd.md)
- Current `gencsv` README: `/README.md`
- Current architecture notes: `/CLAUDE.md`
- MySQL `LOAD DATA`: https://dev.mysql.com/doc/refman/8.0/en/load-data.html
- Postgres `COPY`: https://www.postgresql.org/docs/current/sql-copy.html
- SQL Server `BULK INSERT`: https://learn.microsoft.com/sql/t-sql/statements/bulk-insert-transact-sql
- BigQuery `bq load`: https://cloud.google.com/bigquery/docs/loading-data-cloud-storage-parquet
- Spark Parquet: https://spark.apache.org/docs/latest/sql-data-sources-parquet.html
