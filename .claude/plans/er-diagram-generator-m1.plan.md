# Plan: ER-Diagram Generator — M1 (Scanner Port + `er` Subcommand Stub)

**Source PRD**: `.claude/prds/er-diagram-generator.prd.md`
**Selected Milestone**: M1 — Port and clean scanner from `feature/erdiagram`; add `gencsv er` subcommand stub
**Target branch**: `feature/erd-v2` (fresh from `main`)
**Complexity**: Medium

## Summary

Port the salvageable scanner (`src/util/scanner.rs`, 338 lines) from the abandoned `feature/erdiagram` branch into a fresh `feature/erd-v2` branch, hardened to the repo's post-quality-pass standards. Add a `gencsv er <FILE>` clap subcommand that parses CLI args, reads the file, runs the scanner, and prints a token stream (no parser, no generator yet). This milestone produces a working CLI surface and a tested tokenizer ready for M2's parser.

## Patterns to Mirror

| Category | Source | Pattern |
|---|---|---|
| CLI args (derive + short/long + defaults) | `src/main.rs:5-29` | `#[derive(CLAPParser)]` struct, every flag has short + long, defaults via `default_value_t` |
| `run()` orchestrator (lib entrypoint) | `src/lib.rs:9-53` | Public `pub fn run(...) -> RunResult<()>` returning `Box<dyn Error>`; error-context via `.map_err(\|e\| format!("...: {e}"))` |
| Error type alias | `src/lib.rs:7` | `type RunResult<T> = Result<T, Box<dyn Error>>` — repo convention, NOT `anyhow`/`thiserror` (see CLAUDE.md "Error handling") |
| Inline test module (no `#[cfg(test)]`) | `src/util/schema.rs:58-93` | `mod test { #![allow(unused_imports)] use super::*; ... }` per CLAUDE.md convention |
| Integration test via `assert_cmd` | `tests/cli.rs:30-32, 40-48` | `Command::cargo_bin(NAME)? .args(...) .assert() .success() / .failure() .stdout/.stderr(predicate::str::contains(...))` |
| Lazy-initialised regex with `OnceLock` | `src/util/dataframe.rs:14-17`, `src/util/fake.rs:21-24` | `static RE: OnceLock<Regex> = OnceLock::new(); RE.get_or_init(...)` — keep this pattern over `lazy_static`/`once_cell` |
| Sub-module re-export | `src/util/mod.rs` | Add scanner to existing module tree |

## Requirements Restatement

- Cut `feature/erd-v2` from current `main`.
- Salvage **only** `src/util/scanner.rs` from `feature/erdiagram`. The branch's other files regressed code quality and must be ignored.
- Promote scanner to current quality bar: no `unwrap()` outside tests/unreachable paths, `OnceLock`-cached regexes, line-numbered error messages, immutable-by-default style.
- Token model must distinguish at minimum: `Keyword(erDiagram)`, `Ident(String)`, `LBrace`, `RBrace`, `Colon`, `Cardinality(String)` (raw glyph, e.g. `||--o{`), `Newline`, `Comment(String)`. Final shape gets cemented when the parser arrives in M2; M1 only needs a stable enough emission to round-trip the §6 examples from the PRD.
- Add `gencsv er <FILE> [--rows N] [--rows-per K=V]... [--out DIR] [--format csv|parquet]` clap subcommand. Flags must parse correctly but be **stored unused** in M1 (parser/generator land in M2/M3). The only required runtime behaviour: open `<FILE>`, run scanner, print token-per-line debug dump to stdout, exit 0 on clean scan and non-zero with a line-numbered error otherwise.
- Existing flag-based mode (`gencsv -s …`) must continue to work unchanged. Adding a subcommand is a breaking change to clap structure unless we use a default-subcommand pattern — see "Risks" below for the chosen approach.
- Two integration test fixtures: one valid `.mmd` file, one with an unsupported glyph that asserts a line-numbered error.

## Files to Change

| File | Action | Why |
|---|---|---|
| `src/util/scanner.rs` | CREATE | Hardened port of the 338-line scanner from `feature/erdiagram`, with inline tests |
| `src/util/mod.rs` | UPDATE | `pub mod scanner;` |
| `src/main.rs` | UPDATE | Switch from single `Args` struct to `clap::Subcommand` — top-level flat-table args (default) + `er` subcommand |
| `src/lib.rs` | UPDATE | Add `pub fn run_er(file: &str, rows: usize, rows_per: Vec<(String, usize)>, out: PathBuf, format: ErFormat) -> RunResult<()>` that opens file → scans → prints tokens. **Do not** touch existing `run()`. |
| `tests/cli.rs` | UPDATE | Add `test_er_subcommand_help`, `test_er_scans_valid_fixture`, `test_er_rejects_unsupported_glyph` |
| `tests/fixtures/er/car_person.mmd` | CREATE | Minimal valid ER from PRD §9 acceptance criteria |
| `tests/fixtures/er/invalid_glyph.mmd` | CREATE | Diagram with `..` non-identifying relationship — must trigger §7 error |
| `Cargo.toml` | UPDATE (maybe) | Only if scanner needs a crate not already pulled in (likely none — current regex/std is enough). Do NOT bump `polars`, `fake`, `fakeit`, `clap` per CLAUDE.md tech-stack pins. |

## Tasks (TDD order — RED → GREEN → REFACTOR)

### Task 1: Branch + fetch scanner source
- **Action**: `git checkout -b feature/erd-v2`; `git show origin/feature/erdiagram:src/util/scanner.rs > /tmp/scanner_original.rs` for reference (do **not** merge the branch).
- **Mirror**: N/A — branch hygiene.
- **Validate**: `git status` clean on new branch; `/tmp/scanner_original.rs` exists for paste-reference.

### Task 2: Add scanner skeleton with public token enum + failing tests
- **Action**: Create `src/util/scanner.rs` with `pub enum Token { ... }` (variants listed in Requirements above), `pub struct ScanError { pub line: usize, pub message: String }` with `Display`, and `pub fn scan(input: &str) -> Result<Vec<(usize, Token)>, ScanError>` returning `Err(ScanError { line: 0, message: "not implemented".into() })`. Write 6 inline failing tests covering: empty input, missing `erDiagram` header, valid header-only, single entity with attributes, all 8 cardinality glyphs from PRD §6.4, comment-only line.
- **Mirror**: Token-enum style from any `match` arm pattern in the codebase; test style from `src/util/schema.rs:58-93`.
- **Validate**: `cargo test scanner -- --nocapture` — 6 failures, 0 panics.

### Task 3: Implement scanner to pass tests, port from branch source
- **Action**: Port `feature/erdiagram` scanner logic into the skeleton. Rewrite:
  - Replace any `unwrap()` in regex captures with `?`-propagation.
  - Use `OnceLock` for any regexes (mirror `dataframe.rs:14-17`).
  - All errors must include `ScanError::line` (1-based).
  - Strip parameterised types like `varchar(255)` to base name per PRD §6.3.
  - Reject Mermaid `..` non-identifying connector with the exact error string from PRD §7.
- **Mirror**: `src/util/fake.rs:21-24` for `OnceLock<Regex>`; `src/util/dataframe.rs:64-102` for `?`-propagation with `format!("...: {e}")` context.
- **Validate**: `cargo test scanner` — all green. `cargo clippy --all-targets -- -D warnings` — no warnings.

### Task 4: Wire `er` subcommand into clap
- **Action**: Refactor `src/main.rs`:
  ```rust
  #[derive(CLAPParser)]
  struct Cli {
      #[command(subcommand)]
      command: Option<Command>,
      #[command(flatten)]
      flat: FlatArgs,   // existing flat-table flags
  }

  #[derive(Subcommand)]
  enum Command {
      /// Generate relationally-consistent multi-table data from a Mermaid ER diagram
      Er(ErArgs),
  }
  ```
  Dispatch: `Some(Er(a)) => gencsv::run_er(...)`, `None => gencsv::run(...)` (existing behaviour preserved).
- **Mirror**: `src/main.rs:5-29` for short+long flag style on `ErArgs`; defaults via `default_value_t`.
- **Validate**: `cargo build --release` succeeds. `cargo run -- --help` shows both flat-table flags and the `er` subcommand. `cargo run -- -s "a:INT" -r 3` still works (regression check).

### Task 5: Add `run_er` orchestrator in lib
- **Action**: In `src/lib.rs`, add `pub fn run_er(file: &str, rows: usize, rows_per: Vec<(String, usize)>, out: PathBuf, format: ErFormat) -> RunResult<()>`. M1 body: `std::fs::read_to_string(file).map_err(...)?` → `scanner::scan(&contents).map_err(\|e\| format!("{file}:{}: {}", e.line, e.message))?` → `for (line, tok) in tokens { println!("{line}\t{tok:?}"); }`. Leave `rows`, `rows_per`, `out`, `format` parsed-but-unused (mark with `_` prefix only if clippy complains — preferable to keep names for M2 readers).
- **Mirror**: `src/lib.rs:9-53` for error-context pattern; `RunResult<T>` alias already exists.
- **Validate**: `cargo run -- er tests/fixtures/er/car_person.mmd` prints tokens line-by-line; exit code 0.

### Task 6: Integration tests
- **Action**: Append three tests to `tests/cli.rs`:
  ```rust
  #[test] fn test_er_subcommand_help() // asserts "er" in --help output
  #[test] fn test_er_scans_valid_fixture() // success + stdout contains "erDiagram"
  #[test] fn test_er_rejects_unsupported_glyph() // failure + stderr contains "non-identifying" + line number
  ```
  Use existing `assert_cmd` + `predicates` patterns from `tests/cli.rs:40-48`.
- **Mirror**: `tests/cli.rs:40-48` (negative path with `.failure()` + `.stderr(predicate::str::contains(...))`).
- **Validate**: `cargo test` — all green including the existing 10+ tests.

### Task 7: Quality gates + commit
- **Action**: Run the full local pre-commit suite. Fix anything red. Then conventional commit.
- **Validate**: see Validation block below.

## Validation

```bash
# Formatting
cargo fmt --all -- --check

# Lints (CLAUDE.md notes CI doesn't run clippy, but local pre-commit per global rules does)
cargo clippy --all-targets -- -D warnings

# Full test suite (unit + integration)
cargo test

# Smoke test the new subcommand manually
cargo run --release -- er tests/fixtures/er/car_person.mmd

# Regression: existing flat-table mode unchanged
cargo run --release -- -s "id:INT_INC,name:STRING" -r 5
cargo run --release  # default schema, 10 rows
```

Coverage (`cargo llvm-cov --fail-under-lines 80`) is listed as an M5 acceptance criterion in the PRD, not M1 — CLAUDE.md notes the repo has no coverage tooling wired up yet. Skip the coverage check in M1; it lands in M5.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Clap subcommand refactor breaks existing flag-based mode | **High** | Use `Option<Subcommand>` + `#[command(flatten)] FlatArgs` so `gencsv -s ... -r 3` still parses with no subcommand. Add a regression integration test before refactor. |
| Salvaged scanner has hidden `unwrap()` panics on malformed input | Medium | Task 2 writes failing tests for malformed input first; Task 3 cannot pass without `?`-propagation throughout. |
| Token enum design choices in M1 prove wrong for M2 parser | Medium | Keep `Token` `pub(crate)`, not `pub`, so M2 can refactor freely. Scope M1 to round-tripping the PRD §6 examples; resist designing for the parser. |
| Scanner port pulls in a new crate not on the CLAUDE.md pin list | Low | Audit `feature/erdiagram` scanner imports before porting; reject any new dep unless justified in a separate PR. Current code already has `regex` + `std`. |
| Fixture path conventions differ from existing `tests/expected/` style | Low | New `tests/fixtures/er/` subdir is additive; doesn't conflict with golden-file `tests/expected/` files. |
| `ErFormat` enum design creep (parquet support, etc.) | Low | M1 only needs the flag to parse. Implement as `#[derive(clap::ValueEnum)] enum ErFormat { Csv, Parquet }` with `default = Csv`; real wiring is M3. |

## Acceptance

- [ ] `feature/erd-v2` branch cut fresh from `main`
- [ ] `src/util/scanner.rs` exists with `pub enum Token`, `pub fn scan`, inline tests, no `unwrap()` outside tests
- [ ] `gencsv er <FILE>` parses and dumps tokens line-by-line
- [ ] `gencsv -s "..." -r N` continues to work unchanged (regression-tested)
- [ ] `gencsv er tests/fixtures/er/invalid_glyph.mmd` exits non-zero with `line N: non-identifying relationships ('..') not supported; use '--'`
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` all green
- [ ] No new crate added to `Cargo.toml` (scanner uses only existing deps)
- [ ] M1 row in PRD §8 marked `in-progress` with link to this plan

## Out of Scope (deferred to later milestones)

- ER AST construction → **M2**
- Topological generation, FK wiring, `MultiFileSink` → **M3**
- M:N junction tables → **M4**
- `docs/ERD.md`, coverage gate → **M5**
- Any `--target <dialect>` work (see sibling PRD `db-target-types.prd.md`, sequenced after M5)
