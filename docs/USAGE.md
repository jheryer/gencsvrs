# Using gencsv

This guide goes deeper than the README. It assumes you already have `gencsv`
installed (`cargo install --path .` from the repo root).

## Table of contents

- [Mental model](#mental-model)
- [Choosing an output sink](#choosing-an-output-sink)
- [Cookbook](#cookbook)
- [Schemas in practice](#schemas-in-practice)
- [Reproducibility and seeding](#reproducibility-and-seeding)
- [Error messages you might see](#error-messages-you-might-see)
- [Extending gencsv](#extending-gencsv)

---

## Mental model

A single `gencsv` invocation is one pipeline:

```
(schema) -> generate rows -> [optional append] -> [optional delete] -> sink
```

Each stage is independent. The schema defines the columns; the generator fills
`--rows` of them; `--append-target` concatenates an existing Parquet file in
front of the generated rows; `--delete-target` removes rows from the combined
frame by index; finally the sink (stdout, CSV file, or Parquet file) writes
the result.

If you understand that ordering, every CLI option falls into place.

---

## Choosing an output sink

| You want…                          | Flags                                  |
|------------------------------------|----------------------------------------|
| CSV on stdout (default)            | *(no flags)*                           |
| CSV to a file                      | `-c -f data.csv`                       |
| Parquet to a file                  | `-p -f data.parquet`                   |
| Parquet on stdout                  | **Not supported.** Always needs `-f`.  |

`gencsv` will refuse to run with `-p` and no `-f`; this avoids the
silent-discard footgun that earlier versions had.

---

## Cookbook

### Seed a Postgres table

```sh
gencsv -s "id:INT_INC,email:STRING,signup:DATE" -r 5000 -c -f users.csv
psql -c "\copy users(id,email,signup) FROM 'users.csv' WITH (FORMAT csv, HEADER true)"
```

### Build a Parquet fixture for a pyarrow test

```sh
gencsv -s "ticker:STATE_ABBR,price:PRICE,ts:DATE_TIME" -r 100000 \
       -p -f tests/fixtures/prices.parquet
```

### Incremental fixture growth

You have `prices.parquet` from yesterday and want to append today's batch:

```sh
gencsv -s "ticker:STATE_ABBR,price:PRICE,ts:DATE_TIME" -r 1000 \
       -p -f prices.parquet -a prices.parquet
```

The schema you pass with `-s` must match the schema of `-a`. Polars will
surface a clear error if they don't.

### Mass-edit rows on the way out

Generate a fixture, then drop every other row out of the first 10:

```sh
gencsv -s "id:INT_INC,note:LOREM_WORD" -r 50 \
       -d "0,2,4,6,8"
```

Negative-range deletes start with `-`, so use the `=` form to keep clap from
treating the leading dash as a short flag:

```sh
gencsv -s "id:INT_RNG:(-5-10),note:STRING" -r 16 --delete-target=-2-2
```

---

## Schemas in practice

A schema is just a comma-separated list of `name:TYPE[:(modifier)]` tokens.
Whitespace around tokens is stripped, so these are equivalent:

```text
id:INT_INC,name:NAME
id : INT_INC , name : NAME
```

### Modifiers

Right now only `INT_RNG` accepts a modifier:

```text
id:INT_RNG:(0-100)       # sequential ints starting at 0
score:INT_RNG:(-50-50)   # negative starting points are fine
bad:INT_RNG              # missing modifier -> warning, falls back to (0-rows)
bad2:INT_RNG:(garbage)   # malformed modifier -> warning, falls back to (0-rows)
```

When `INT_RNG`'s modifier is missing or malformed, `gencsv` writes a warning
to stderr and uses `(0-rows)` so the run still completes. If you want strict
parsing, grep for `INT_RNG` in stderr and fail your build.

### Invalid columns

Columns that don't parse as `name:TYPE` or `name:TYPE:(mod)` are skipped:

```text
id:INT_INC, , name:NAME
              ^ skipped, warning on stderr
```

If **every** column is invalid, `gencsv` exits non-zero with a message
naming the expected format.

### Unknown types

Unknown type names fall through to the literal string `"unknown"`. This makes
typos easy to spot:

```sh
gencsv -s "id:INT_INC,name:NMAE" -r 3
# id,name
# 0,unknown
# 1,unknown
# 2,unknown
```

---

## Reproducibility and seeding

`gencsv` does **not** seed its random number generator. Two runs with the
same flags produce different data for every type except:

- `INT_INC` — always `0..rows`
- `INT_RNG` — sequential starting at `lower`
- The default `VALUE` placeholder (literal `"value"`)

If you need a deterministic fixture, build it once and commit the resulting
CSV or Parquet file rather than re-running `gencsv` in CI.

---

## Error messages you might see

| Message                                                                              | Cause / fix |
|--------------------------------------------------------------------------------------|-------------|
| `--parquet output requires --file-target <PATH>; refusing to discard generated rows` | You passed `-p` without `-f`. Add a file path. |
| `schema string '...' produced no valid columns; expected 'name:TYPE[,name:TYPE...]'` | Every column in `-s` was malformed. Re-check the syntax. |
| `failed to open parquet file 'X': No such file or directory`                         | `--append-target` points at a missing file. |
| `failed to read parquet file 'X': ...`                                               | The file exists but isn't valid Parquet. |
| `failed to append generated rows to 'X': schemas do not match`                        | Your `-s` schema differs from the schema of the file you're appending to. |
| `INT_RNG column 'foo' has no (lo-hi) modifier; using default range`                  | Warning only; column still produced. |
| `ignoring invalid schema column: [...]`                                              | One column token didn't parse; the rest of the schema ran. |

---

## Extending gencsv

If you want to add a new data type:

1. Add a `pub fn fake_xxx() -> String` (or appropriate return type) in
   [`src/util/fake.rs`](../src/util/fake.rs).
2. Add a matching arm to the `create_column` dispatch in the same file.
3. Add the new type to the table in [README.md](../README.md#data-types).
4. Add at least one unit test asserting non-panic behaviour for both the
   plain and (if applicable) modifier-bearing forms.

The internal architecture is intentionally small:

```
src/
├── main.rs            -> clap Args, calls lib::run
├── lib.rs             -> public run() orchestrator
└── util/
    ├── schema.rs      -> Schema struct, parse_schema, default_schema
    ├── fake.rs        -> per-type generators + create_column dispatch
    ├── dataframe.rs   -> create_dataframe, append + delete
    └── output.rs      -> Output trait + Console / CSVFile / ParquetFile
```

New output formats slot into `output.rs` by implementing the `Output` trait
and matching them in `run()`.
