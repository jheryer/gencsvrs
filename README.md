# gencsv

A small Rust CLI that generates realistic-looking fake tabular data and writes
it to stdout, CSV, or Parquet. Useful for seeding databases, smoke-testing data
pipelines, and producing fixture files without spinning up a generator service.

```
$ gencsv -s "id:INT_INC,name:NAME,email:STRING,joined:DATE" -r 5
id,name,email,joined
0,Mariana Stehr,abc...,2168-09-12
1,Otho Becker,def...,1789-04-07
2,Lyric Reichel,ghi...,2102-11-30
3,Glen Yundt,jkl...,1450-06-22
4,Lavada Schmidt,mno...,2521-02-18
```

- **Schema-driven**: declare columns inline, no config file.
- **Three sinks**: stdout (default), `.csv`, or `.parquet`.
- **Compose with existing data**: append generated rows to an existing Parquet
  file and optionally drop rows by index.

For deeper usage, see [docs/USAGE.md](docs/USAGE.md). For all available data
types and their modifiers, see [Data Types](#data-types) below. For multi-table
relational data generation see [gencsv er](#gencsv-er--relational-data-from-an-er-diagram).

---

## Install

Requires a recent stable Rust toolchain (`rustup`).

```sh
git clone git@github.com:jheryer/gencsvrs.git
cd gencsvrs
cargo install --path .
```

The binary is `gencsv` (not `gencsvrs`).

---

## Quick start

```sh
# 10 rows of the default 4-column placeholder schema, printed to stdout
gencsv

# Custom schema, custom row count, CSV to stdout
gencsv -s "id:INT_INC,name:NAME,phone:PHONE" -r 25

# Write to a CSV file
gencsv -s "id:INT_INC,city:STATE_NAME" -r 1000 -c -f cities.csv

# Write to a Parquet file
gencsv -s "id:INT_INC,price:PRICE" -r 1000 -p -f prices.parquet

# Append 5 new rows to an existing parquet file, then drop rows 0..2
gencsv -s "id:INT_INC,price:PRICE" -r 5 -p -f out.parquet \
       -a prices.parquet --delete-target=0-2
```

---

## CLI reference

```text
Usage: gencsv [OPTIONS]

Options:
  -s, --schema <SCHEMA>                Column definitions, e.g.
                                       "id:INT_INC,name:NAME,date:DATE".
                                       If omitted, a 4-column placeholder
                                       schema is used.
  -r, --rows <ROWS>                    Number of rows to generate [default: 10]
  -c, --csv                            Force CSV output (default)
  -p, --parquet                        Parquet output. Requires --file-target.
  -f, --file-target <FILE_TARGET>      Output file path. Without this, output
                                       goes to stdout (CSV only).
  -a, --append-target <APPEND_TARGET>  Path to an existing parquet file. Newly
                                       generated rows are appended to its
                                       contents. Schemas must match.
  -d, --delete-target <DELETE_TARGET>  Rows to drop after generation. Accepts:
                                         a single integer (e.g. `3`),
                                         a comma list (`0,2,5`),
                                         an inclusive range (`0-9`, `-3-3`),
                                         the literal `random` / `rand`.
                                       Use `--delete-target=-2-2` (with `=`)
                                       for ranges that start with a negative.
      --target <DIALECT>               Emit DDL + load-command files next to the
                                       data file. Values: mysql, postgres,
                                       sqlserver, bigquery, spark. Requires -f.
      --no-ddl                         Suppress DDL file when --target is set
      --no-load                        Suppress load-command file when --target
                                       is set
  -h, --help                           Print help
  -V, --version                        Print version
```

### Default behaviour

- If neither `-c` nor `-p` is set, gencsv emits CSV.
- If `-c` is set without `-f`, CSV is written to stdout.
- If `-p` is set, `-f` is **required** — gencsv will exit non-zero rather than
  generate and silently discard data.
- If `-s` is omitted, a 4-column literal-value schema (`col1..col4`, all the
  text `"value"`) is used. This exists so the default invocation has
  deterministic output.

---

## Schema format

Schemas are a comma-separated list of column definitions:

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

- Whitespace around tokens is stripped.
- Columns that don't match `name:TYPE` or `name:TYPE:(mod)` are dropped with a
  warning on stderr; the rest of the schema still runs.
- Unknown types fall through to the literal string `"unknown"`.

---

## Data types

Plain types (no modifier):

| Type              | Output                                              |
|-------------------|-----------------------------------------------------|
| `STRING`          | Faker-generated free-form ASCII string              |
| `INT`             | Random `i32` in `[0, i32::MAX)`                     |
| `INT_INC`         | Sequential integer starting at 0                    |
| `DIGIT`           | Single decimal digit as a string                    |
| `DECIMAL`         | Random `f32` in `[0.0, 100000.0)`                   |
| `DATE`            | Random calendar date                                |
| `TIME`            | Random wall-clock time                              |
| `DATE_TIME`       | Random combined date+time                           |
| `NAME`            | Full personal name (en)                             |
| `FIRST_NAME`      | First name only                                     |
| `LAST_NAME`       | Last name only                                      |
| `SSN`             | US-style fake SSN                                   |
| `ZIP_CODE`        | US-style postal code                                |
| `COUNTRY_CODE`    | ISO-style country code                              |
| `STATE_NAME`      | US state name                                       |
| `STATE_ABBR`      | US state abbreviation                               |
| `LAT` / `LON`     | Latitude / longitude as decimal string              |
| `PHONE`           | US-formatted phone number                           |
| `PRICE`           | Currency-formatted amount in `[0.00, 9999.00]`      |
| `LOREM_WORD`      | One lorem-ipsum word                                |
| `LOREM_TITLE`     | 1–3 capitalized lorem words                         |
| `LOREM_SENTENCE`  | Lorem sentence                                      |
| `LOREM_PARAGRAPH` | Lorem paragraph                                     |
| `UUID`            | RFC 4122 v4 UUID                                    |

Types that accept a modifier:

| Type      | Modifier syntax | Example         | Behaviour |
|-----------|-----------------|-----------------|-----------|
| `INT_RNG` | `(lower-upper)` | `(-15-23)`      | Sequential integers starting at `lower`. If the modifier is missing or malformed, falls back to `(0-rows)` with a stderr warning. |

---

## Append + delete semantics

- `--append-target FILE` reads `FILE` as Parquet, generates new rows from the
  current `--schema`, and emits the **combined** frame. The two schemas must be
  compatible; otherwise gencsv exits with a polars error message.
- `--delete-target` runs **after** append. Indexes refer to the row positions
  of the combined frame, not just the newly generated rows.
- Negative-range deletes (e.g. `-3-3`) require the `=` form
  (`--delete-target=-3-3`) so that clap does not treat the leading dash as a
  short flag.
- `random` / `rand` deletes a random number of random row indexes (handy for
  fuzz fixtures).

---

## gencsv er — relational data from an ER diagram

Generate one CSV (or Parquet) file per entity — plus junction tables for M:N
relationships — with FK columns automatically populated from the parent's PK.

```sh
gencsv er schema.mmd -r 100 --out ./data
```

Write a [Mermaid `erDiagram`](https://mermaid.js.org/syntax/entityRelationshipDiagram.html)
file and pass it to `gencsv er`. Every entity block becomes a file; every
relationship wires the FK column automatically.

```
erDiagram
    CUSTOMER {
        int    id   PK
        string name
    }
    ORDER {
        int id PK
        int customer_id FK
    }
    CUSTOMER ||--o{ ORDER : places
```

```sh
gencsv er shop.mmd -r 500 --rows-per ORDER=2000 --out ./data
# data/CUSTOMER.csv  — 500 rows
# data/ORDER.csv     — 2 000 rows, customer_id ∈ CUSTOMER.id
```

**Flags:**

| Flag | Default | Description |
|---|---|---|
| `--out` / `-o` | `./out` | Output directory (created if needed) |
| `--rows` / `-r` | `10` | Default rows per entity |
| `--rows-per ENTITY=N` | — | Per-entity override (repeatable) |
| `--format` / `-F` | `csv` | `csv` or `parquet` |
| `--target <DIALECT>` | — | Emit DDL + load-command files (see below) |
| `--no-ddl` | — | Suppress DDL file when `--target` is set |
| `--no-load` | — | Suppress load-command file when `--target` is set |

See [docs/ERD.md](docs/ERD.md) for supported types, glyph reference, and validation rules.

---

## Database DDL and load commands

Pass `--target <dialect>` to emit a `CREATE TABLE` DDL file and a dialect-specific
load-command snippet alongside your generated data.

```sh
# Flat mode — writes users.csv, users.ddl.postgres.sql, users.load.postgres.sql
gencsv -s "id:INT_INC,name:NAME,email:STRING" -r 1000 -c -f users.csv \
       --target postgres

# ER mode — writes schema.ddl.mysql.sql + per-entity .load.mysql.sql files
gencsv er shop.mmd -r 500 --out ./data --target mysql
```

Supported targets: `mysql`, `postgres`, `sqlserver`, `bigquery`, `spark`.

See [docs/DIALECTS.md](docs/DIALECTS.md) for the full type-mapping table, load-command
templates, and Parquet logical-type guidance for BigQuery and Spark.

---

## Development

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs all three on push and PR.

---

## License

MIT — see [LICENSE](LICENSE).
