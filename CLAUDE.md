# CLAUDE.md

Project-specific notes for working in this repo. General Rust/coding rules live in `~/.claude/rules/` — only items below this line are non-obvious or repo-specific.

## Identity

- Repo dir: `synthtab` (formerly `gencsvrs`). Cargo crate + binary: **`synthtab`**. `cargo install --path .` produces `synthtab`. Historical note: the binary was `gencsv` before v0.2.0.
- Single-binary CLI; library entry point is `synthtab::run(...)` in `src/lib.rs`.

## Tech stack pins (do not bump casually)

| Crate | Version | Notes |
|---|---|---|
| `polars` | **0.38.3** | Old. APIs used here (`Series::new(&str, Vec<T>)`, `DataFrame::new(Vec<Series>)`, `ParquetWriter::new().finish()`, `CsvWriter::new().finish()`) changed in 0.39+. Any upgrade is a rewrite, not a bump. |
| `fake` | 2.9.2 | Used for most generators (strings, names, dates, addresses, lorem, phones). |
| `fakeit` | 1.2.0 | Used **only** for `FIRST_NAME`, `LAST_NAME`, `SSN`, `PRICE` (currency). Both fake libs coexist on purpose. |
| `clap` | 4.5 derive | All flags are short + long; defaults handled in `Args` struct. |

Features enabled on polars: `lazy`, `parquet`, `csv`. Don't drop any — `filter_by_index` uses the lazy API.

## CLI shape (gotchas)

- Flag `-d / --delete-target` deletes **rows by index**, not a delimiter. Don't confuse with the commented-out test in `tests/cli.rs` that references `-d "|"` (that test is for a never-built pipe-delimiter feature).
- If neither `-c` nor `-p` is passed, `run()` flips `csv = true`. Parquet without `-f` is silently a no-op (see `src/lib.rs:51`).
- Schema flag accepts `col:TYPE` or `col:TYPE:(modifier)`. Modifier is parenthesized, e.g. `INT_RNG:(-15-23)`. Spaces inside the schema string are stripped via `replace(" ", "")` in `Schema::from_string`.
- `-d` (delete) accepts: single int (`3`), comma list (`0,2,5`), inclusive range (`0-2`), or literal `random` / `rand`.

## Data-type registry

The full type list lives in **three places that must stay in sync**:
1. `match` arm in `create_column` — `src/util/fake.rs:46`
2. Corresponding `pub fn fake_xxx() -> ...` generator below it
3. README "Available Data Types" section

Unknown types fall through to the literal string `"unknown"` (see `unknown_string`). `VALUE` returns the literal string `"value"` — this exists so golden-file tests can be deterministic; don't "fix" it.

## Test conventions

- Integration tests in `tests/cli.rs` use `assert_cmd` to exec the built binary and diff stdout against files in `tests/expected/`.
- **Only the default schema is golden-file testable** because every other type is non-deterministic (no seeded RNG). Don't add stdout-diff tests for `NAME`, `DATE`, etc.
- Inline test modules are written as bare `mod test { #![allow(unused_imports)] use super::*; ... }` — **no `#[cfg(test)]` attribute** on the module. This is deliberate in the existing files; follow the same style when adding tests so warnings stay quiet.
- Commented-out tests in `tests/cli.rs` reference unbuilt features (`-d "|"` delimiter, `-w parquet` writer flag). Leave them; they document intent.
- No coverage tooling wired up. Don't claim 80% coverage from the global rules — this repo doesn't measure it.

## Error handling

Codebase uses `Box<dyn Error>` everywhere (`type RunResult<T>`, `type DataFrameResult`, `type DeleteTargetResult`) and `.unwrap()` liberally on polars calls. Global rules say "no `unwrap` in production / use `thiserror`/`anyhow`" — **this repo predates that rule**. Match local style for small changes; only introduce `anyhow`/`thiserror` if doing a broader cleanup pass and the user asks for it.

## CI

`.github/workflows/rust.yml` runs only `cargo build --verbose` and `cargo test --verbose` on push/PR to `main`. No clippy, no fmt check. Local pre-commit per global rules (`cargo fmt`, `cargo clippy -- -D warnings`) is still expected before pushing.

## Module layout

```
src/
├── main.rs            # clap Args + call into lib::run
├── lib.rs             # public run() orchestrator
└── util/
    ├── mod.rs         # re-exports submodules
    ├── schema.rs      # Schema struct + parse_schema / default_schema
    ├── fake.rs        # create_column dispatch + per-type generators
    ├── dataframe.rs   # create_dataframe, append/delete, filter_by_index
    └── output.rs      # Output trait + Console / CSVFile / ParquetFile / MockConsole
```

`Output` is a trait deliberately so new sinks plug in without touching `run()`. `MockConsole` exists for future tests; no current tests use it.

## When adding a new data type

1. Add `pub fn fake_xxx() -> String` (or appropriate type) in `src/util/fake.rs`.
2. Add the match arm in `create_column`.
3. Append the type name to the README's "Available Data Types" list.
4. If the type needs a modifier (like `INT_RNG`), parse it from `element.modifier` and handle the `None` case explicitly — current code `.unwrap()`s, which panics on missing modifier; copy that pattern only if a missing modifier should be a hard error.

## Append + delete semantics

- `--append-target` reads a parquet file and `extend`s newly generated rows onto it. Schemas must match — polars will error otherwise.
- `--delete-target` runs **after** append, on the combined frame. Indexes refer to the combined row positions, not just newly generated ones.
- `filter_by_index` adds a temp column with a UUID name, filters on it, and drops it. The UUID column name avoids collisions with user schema columns — don't replace it with a fixed name.
