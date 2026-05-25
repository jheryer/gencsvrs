# gencsv

A small Rust CLI that generates realistic-looking fake tabular data and writes it to **stdout**, **CSV**, or **Parquet** — optionally with matching **`CREATE TABLE` DDL** and a **load-command** script for your target database.

Two modes:

- **Flat mode** — one schema, one table. Great for fixtures, smoke tests, demo data.
- **ER mode** (`gencsv er`) — generate a whole relational dataset from a [Mermaid `erDiagram`](https://mermaid.js.org/syntax/entityRelationshipDiagram.html) file, with FK values sampled from real parent PKs.

```sh
$ gencsv -s "id:INT_INC,name:NAME,email:STRING,joined:DATE" -r 5
id,name,email,joined
0,Mariana Stehr,abc...,2168-09-12
1,Otho Becker,def...,1789-04-07
2,Lyric Reichel,ghi...,2102-11-30
3,Glen Yundt,jkl...,1450-06-22
4,Lavada Schmidt,mno...,2521-02-18
```

---

## Contents

- [Install](#install)
- [The 60-second tour](#the-60-second-tour)
- [Flat mode examples](#flat-mode-examples)
- [ER mode examples](#er-mode-examples)
- [Database targets — DDL + load commands](#database-targets--ddl--load-commands)
- [CLI reference](#cli-reference)
- [Schema syntax](#schema-syntax)
- [Data types](#data-types)
- [Append + delete semantics](#append--delete-semantics)
- [Gotchas](#gotchas)
- [Development](#development)

---

## Install

Requires a recent stable Rust toolchain (`rustup`).

```sh
git clone git@github.com:jheryer/gencsvrs.git
cd gencsvrs
cargo install --path .
```

The binary is **`gencsv`** (single `s`), not `gencsvrs`.

Verify:

```sh
gencsv --version
```

---

## The 60-second tour

```sh
# 1. Default schema, 10 rows of literal "value" — useful as a sanity check
gencsv

# 2. Custom schema, custom row count, CSV to stdout
gencsv -s "id:INT_INC,name:NAME,phone:PHONE" -r 25

# 3. Write 1,000 rows to a CSV file
gencsv -s "id:INT_INC,city:STATE_NAME" -r 1000 -c -f cities.csv

# 4. Write 1,000 rows to a Parquet file
gencsv -s "id:INT_INC,price:PRICE" -r 1000 -p -f prices.parquet

# 5. Generate CSV + a Postgres CREATE TABLE + a \copy snippet in one shot
gencsv -s "id:INT_INC,email:STRING,joined:DATE" -r 500 \
       -c -f users.csv --target postgres
# → users.csv
# → users.ddl.postgres.sql
# → users.load.postgres.sql

# 6. Generate a whole relational dataset from an ER diagram
gencsv er shop.mmd -r 500 --rows-per ORDER=2000 --out ./data --target mysql
```

---

## Flat mode examples

### Pipe straight into another tool

```sh
# Count generated rows
gencsv -s "id:INT_INC" -r 1000000 | wc -l

# Load directly into DuckDB
gencsv -s "id:INT_INC,price:PRICE,ts:DATE_TIME" -r 50000 \
  | duckdb -c "CREATE TABLE events AS SELECT * FROM read_csv_auto('/dev/stdin');"

# Pipe into psql
gencsv -s "id:INT_INC,email:STRING" -r 10000 \
  | psql -c "\copy users(id,email) FROM STDIN WITH (FORMAT csv, HEADER true)"
```

### A user-signup fixture

```sh
gencsv -s "id:INT_INC,first:FIRST_NAME,last:LAST_NAME,email:STRING,signup:DATE_TIME" \
       -r 10000 -c -f users.csv
```

### A geo-tagged events Parquet for pyarrow / Spark / Polars tests

```sh
gencsv -s "id:UUID,lat:LAT,lon:LON,city:STATE_NAME,observed:DATE_TIME" \
       -r 100000 -p -f tests/fixtures/events.parquet
```

### Sequential integers in a range (signed lower bound is fine)

```sh
gencsv -s "id:INT_INC,delta:INT_RNG:(-50-50)" -r 100 -c -f range.csv
```

### Currency + decimals for a billing fixture

```sh
gencsv -s "invoice_id:UUID,amount:PRICE,tax:DECIMAL,paid_on:DATE" \
       -r 5000 -c -f invoices.csv
```

### Mixed lorem text for content placeholders

```sh
gencsv -s "id:INT_INC,title:LOREM_TITLE,body:LOREM_PARAGRAPH,slug:LOREM_WORD" \
       -r 250 -c -f posts.csv
```

### Append today's batch onto an existing Parquet file

```sh
# yesterday's file: prices.parquet (same schema)
gencsv -s "ticker:STATE_ABBR,price:PRICE,ts:DATE_TIME" -r 1000 \
       -p -f prices.parquet -a prices.parquet
```

The combined frame (yesterday + today) is written back to `prices.parquet`. Schemas must match.

### Generate, then drop rows by index

```sh
# Drop a single row
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 50 -d 7

# Drop rows 0–9
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 50 -d 0-9

# Drop a comma list
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 50 -d 0,2,4,6,8

# Drop a random subset (count + indexes both randomized)
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 50 -d random

# Negative-starting range needs the = form so clap doesn't read the leading - as a flag
gencsv -s "id:INT_RNG:(-5-10),note:STRING" -r 16 --delete-target=-2-2
```

---

## ER mode examples

`gencsv er` reads a Mermaid `erDiagram`, writes one file per entity, and **auto-wires FK columns** by sampling from the parent's PK column — so the dataset is referentially consistent out of the box.

### Minimal: customers + orders

`shop.mmd`:

```
erDiagram
    CUSTOMER {
        int    id   PK
        string name
        string email
    }
    ORDER {
        int      id          PK
        int      customer_id FK
        datetime placed_at
        decimal  total
    }
    CUSTOMER ||--o{ ORDER : places
```

```sh
gencsv er shop.mmd -r 500 --rows-per ORDER=2000 --out ./data
# → data/CUSTOMER.csv   (500 rows)
# → data/ORDER.csv      (2 000 rows, customer_id ∈ CUSTOMER.id)
```

### Parquet output

```sh
gencsv er shop.mmd -r 500 --out ./data -F parquet
# → data/CUSTOMER.parquet
# → data/ORDER.parquet
```

### Many-to-many produces a junction table

`enrollments.mmd`:

```
erDiagram
    STUDENT  { int id PK   string name }
    COURSE   { int id PK   string title }
    STUDENT }o--o{ COURSE : enrolled_in
```

```sh
gencsv er enrollments.mmd -r 200 --rows-per STUDENT_COURSE=5000 --out ./data
# → data/STUDENT.csv
# → data/COURSE.csv
# → data/STUDENT_COURSE.csv  (junction table with both FKs)
```

### Three-level hierarchy

```
erDiagram
    REGION   { int id PK   string name }
    STORE    { int id PK   int region_id FK   string name }
    PRODUCT  { int id PK   int store_id FK    string name   decimal price }
    REGION ||--o{ STORE   : has
    STORE  ||--o{ PRODUCT : sells
```

```sh
gencsv er retail.mmd \
       -r 10 \
       --rows-per STORE=200 \
       --rows-per PRODUCT=10000 \
       --out ./data
```

Every `region_id` in `STORE.csv` exists in `REGION.csv`; every `store_id` in `PRODUCT.csv` exists in `STORE.csv`.

### Per-entity row counts cheat sheet

| Flag | Effect |
|---|---|
| `-r 100` | Default for every entity that isn't overridden |
| `--rows-per ORDER=2000` | Override one entity |
| `--rows-per A=100 --rows-per B=500` | Repeatable for multiple entities |
| `--rows-per STUDENT_COURSE=5000` | Override a junction table by name (`LEFT_RIGHT`) |

See [docs/ERD.md](docs/ERD.md) for the full Mermaid syntax, glyph reference, validation rules, and type mapping.

---

## Database targets — DDL + load commands

Add `--target <dialect>` to **flat mode** or **ER mode** and you'll get a matching `CREATE TABLE` script plus a load snippet beside your data.

### Postgres (flat mode)

```sh
gencsv -s "id:INT_INC,name:NAME,email:STRING,joined:DATE" -r 1000 \
       -c -f users.csv --target postgres
```

Produces:

- `users.csv`
- `users.ddl.postgres.sql` — `CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, email TEXT, joined DATE);`
- `users.load.postgres.sql` — `\copy users FROM 'users.csv' WITH (FORMAT csv, HEADER true);`

Run it:

```sh
psql mydb -f users.ddl.postgres.sql
psql mydb -f users.load.postgres.sql
```

### MySQL (ER mode)

```sh
gencsv er shop.mmd -r 500 --out ./data --target mysql
# → data/CUSTOMER.csv, data/ORDER.csv
# → data/schema.ddl.mysql.sql       (CREATE TABLE for all entities + FK constraints)
# → data/CUSTOMER.load.mysql.sql    (LOAD DATA LOCAL INFILE)
# → data/ORDER.load.mysql.sql
```

### SQL Server

```sh
gencsv -s "id:INT_INC,sku:UUID,price:PRICE" -r 5000 \
       -c -f products.csv --target sqlserver
# → products.ddl.sqlserver.sql  (INT IDENTITY(1,1) PRIMARY KEY, UNIQUEIDENTIFIER, DECIMAL(10,2))
# → products.load.sqlserver.sql (BULK INSERT)
```

### BigQuery

```sh
gencsv -s "id:INT_INC,event:STRING,ts:DATE_TIME" -r 50000 \
       -p -f events.parquet --target bigquery
# → events.parquet
# → events.ddl.bigquery.sql      (INT64, STRING, DATETIME)
# → events.load.bigquery.sh      (bq load --source_format=PARQUET ...)
```

### Spark / Databricks

```sh
gencsv er events.mmd -r 1000 --out ./data -F parquet --target spark
# → data/*.parquet
# → data/schema.ddl.spark.sql
# → data/<entity>.load.spark.py  (PySpark spark.read snippets)
```

### Suppress one or the other

```sh
gencsv -s "..." -r 100 -c -f x.csv --target postgres --no-load  # DDL only
gencsv -s "..." -r 100 -c -f x.csv --target postgres --no-ddl   # load script only
```

Full type-mapping table and Parquet logical-type guidance: [docs/DIALECTS.md](docs/DIALECTS.md).

---

## CLI reference

```text
gencsv [OPTIONS]                # flat mode
gencsv er <SCHEMA.mmd> [OPTS]   # ER mode
```

### Flat mode flags

| Flag | Default | Description |
|---|---|---|
| `-s, --schema <SCHEMA>` | 4-column literal-value schema | Column definitions, e.g. `"id:INT_INC,name:NAME"` |
| `-r, --rows <N>` | `10` | Number of rows to generate |
| `-c, --csv` | on if neither `-c`/`-p` set | Force CSV output |
| `-p, --parquet` | — | Parquet output. **Requires `-f`.** |
| `-f, --file-target <PATH>` | — | Output file path. Without it, output goes to stdout (CSV only). |
| `-a, --append-target <PATH>` | — | Existing Parquet file; generated rows are appended to it |
| `-d, --delete-target <SPEC>` | — | Drop rows by index. See [Append + delete](#append--delete-semantics) |
| `--target <DIALECT>` | — | Emit DDL + load files. `mysql`, `postgres`, `sqlserver`, `bigquery`, `spark`. Requires `-f`. |
| `--no-ddl` | — | Suppress DDL file when `--target` is set |
| `--no-load` | — | Suppress load-command file when `--target` is set |
| `-h, --help` | — | Print help |
| `-V, --version` | — | Print version |

### ER mode flags

| Flag | Default | Description |
|---|---|---|
| `<SCHEMA.mmd>` | — | Path to a Mermaid `erDiagram` file (positional) |
| `-r, --rows <N>` | `10` | Default rows per entity |
| `--rows-per <NAME>=<N>` | — | Per-entity override (repeatable) |
| `-o, --out <DIR>` | `./out` | Output directory (created if needed) |
| `-F, --format <FMT>` | `csv` | `csv` or `parquet` |
| `--target <DIALECT>` | — | Emit DDL + per-entity load files |
| `--no-ddl` | — | Suppress DDL file |
| `--no-load` | — | Suppress load-command files |

---

## Schema syntax

Comma-separated list of column definitions:

```
<column_name>:<TYPE>[:(<modifier>)]
```

Examples:

```text
id:INT_INC
name:STRING
range:INT_RNG:(-15-23)
price:PRICE
```

Rules:

- Whitespace around tokens is stripped (`id : INT_INC , name : NAME` works).
- Columns that don't match `name:TYPE` or `name:TYPE:(mod)` are **skipped with a warning** on stderr; the rest of the schema still runs.
- If **every** column is invalid, gencsv exits non-zero.
- Unknown types fall through to the literal string `"unknown"` — typos are obvious.

---

## Data types

### Plain types (no modifier)

| Type | Output |
|---|---|
| `STRING` | Faker-generated free-form ASCII string |
| `INT` | Random `i32` in `[0, i32::MAX)` |
| `INT_INC` | Sequential integer starting at 0 |
| `DIGIT` | Single decimal digit as a string |
| `DECIMAL` | Random `f32` in `[0.0, 100000.0)` |
| `DATE` | Random calendar date |
| `TIME` | Random wall-clock time |
| `DATE_TIME` | Random combined date+time |
| `NAME` | Full personal name (en) |
| `FIRST_NAME` | First name only |
| `LAST_NAME` | Last name only |
| `SSN` | US-style fake SSN |
| `ZIP_CODE` | US-style postal code |
| `COUNTRY_CODE` | ISO-style country code |
| `STATE_NAME` | US state name |
| `STATE_ABBR` | US state abbreviation |
| `LAT` / `LON` | Latitude / longitude as decimal string |
| `PHONE` | US-formatted phone number |
| `PRICE` | Currency-formatted amount in `[0.00, 9999.00]` |
| `LOREM_WORD` | One lorem-ipsum word |
| `LOREM_TITLE` | 1–3 capitalized lorem words |
| `LOREM_SENTENCE` | Lorem sentence |
| `LOREM_PARAGRAPH` | Lorem paragraph |
| `UUID` | RFC 4122 v4 UUID |
| `VALUE` | Literal `"value"` (used by the default schema) |

### Types with a modifier

| Type | Syntax | Example | Behaviour |
|---|---|---|---|
| `INT_RNG` | `(lower-upper)` | `(-15-23)` | Sequential integers starting at `lower`. Missing/malformed modifier → warning + fallback to `(0-rows)`. |

---

## Append + delete semantics

Pipeline order is fixed:

```
(schema) → generate rows → [append] → [delete] → sink
```

- `-a, --append-target FILE` reads `FILE` as Parquet, generates new rows from `-s`, and emits the **combined** frame. Schemas must match — polars will surface a clear error otherwise.
- `-d, --delete-target` runs **after** append. Indexes refer to row positions of the combined frame.
- Accepted delete specs:
  - single: `3`
  - list: `0,2,5`
  - inclusive range: `0-9`
  - negative-starting range: `--delete-target=-3-3` (must use `=` form)
  - random: `random` or `rand` — random count of random indexes

### Worked example

```sh
# Start with a 5-row baseline file
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 5 -p -f notes.parquet

# Append 3 more rows, then drop rows 0 and 7 from the combined 8-row frame
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 3 \
       -p -f notes.parquet -a notes.parquet -d 0,7
```

---

## Gotchas

- **`-d` is delete, not delimiter.** It removes rows by index. There is no pipe-delimiter flag.
- **Parquet always needs `-f`.** `-p` without `-f` exits non-zero (no silent discard).
- **Negative-starting ranges need `=`:** `--delete-target=-2-2`, not `--delete-target -2-2`.
- **No RNG seed.** Reruns differ for every type except `INT_INC`, `INT_RNG`, and `VALUE`. For deterministic fixtures, generate once and commit the artifact.
- **Append schema must match.** Different column names or types → polars error.
- **Unknown types don't error**, they emit the literal `"unknown"`. Look for it in your output to catch typos.
- **ER mode validation is strict.** Unknown glyphs, duplicate entity names, multiple `PK`s per entity, cyclic FKs, and unknown Mermaid types all fail at parse time with a line number.

---

## Development

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs `cargo build` and `cargo test` on push and PR. Local pre-commit (fmt + clippy) is expected before pushing.

Project layout:

```
src/
├── main.rs            # clap Args + entry point
├── lib.rs             # public run() orchestrator
└── util/
    ├── schema.rs      # Schema parsing + default schema
    ├── fake.rs        # Per-type generators + create_column dispatch
    ├── dataframe.rs   # create_dataframe, append/delete, filter_by_index
    └── output.rs      # Output trait + Console / CSVFile / ParquetFile sinks
```

Further reading: [docs/USAGE.md](docs/USAGE.md) · [docs/ERD.md](docs/ERD.md) · [docs/DIALECTS.md](docs/DIALECTS.md).

---

## License

MIT — see [LICENSE](LICENSE).
