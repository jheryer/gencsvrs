# PRD: ER-Diagram-Driven Synthetic Data Generator

**Status:** Draft
**Owner:** jheryer
**Created:** 2026-05-25
**Target branch:** `feature/erd-v2` (fresh from `main`)
**Supersedes:** `feature/erdiagram` (abandoned — pre-quality-pass base)

---

## 1. Problem

`synthtab` today produces a single flat table from a `-s "col:TYPE,…"` schema. Real-world test fixtures need **multiple related tables** where foreign keys actually reference existing primary keys. Hand-wiring this with the current flag-based schema is impossible, and post-generation joining of independent flat CSVs produces broken referential integrity.

## 2. Goal

Generate **relationally-consistent multi-table synthetic data** from a **Mermaid.js `erDiagram` specification**, written to one CSV or Parquet file per entity.

## 3. Non-Goals

- Full Mermaid spec coverage (we ship a strict, documented subset)
- Database connection / direct seeding (use existing CSV/Parquet output)
- Non-relational data (graphs, documents)
- Reproducible runs across invocations (`--seed` deferred to a later release)
- Schema migration / inspection of existing databases
- Reverse engineering (DB → ER diagram)

## 4. User Stories

| As a… | I want to… | So that… |
|---|---|---|
| Backend engineer | Generate 5k customers + 50k orders with valid `customer_id` FKs from one command | I can load realistic fixtures into a staging DB |
| QA engineer | Express my domain as a Mermaid ER file and emit one CSV per table | The same diagram drives both docs and test data |
| Data engineer | Auto-emit a junction table for M:N relationships | I don't have to model link tables manually |
| Anyone | Get a clear, line-numbered error for unsupported Mermaid syntax | I'm not left guessing why my diagram doesn't work |

## 5. Functional Requirements

### 5.1 CLI

New subcommand:

```
synthtab er <FILE>
    [--rows N]                          # default row count per entity (default: 10)
    [--rows-per ENTITY=N]...            # repeatable per-entity overrides
    [--out DIR]                         # output directory (default: ./out)
    [--format csv|parquet]              # default: csv
```

Existing flag-based mode (`synthtab -s …`) stays unchanged and becomes a peer of the `er` subcommand.

### 5.2 Pipeline

```
.mmd file → scanner → parser → ErdAst → validator → topological generator → MultiFileSink
```

### 5.3 Generation rules

- **Topological order**: parents generated before children; cycles rejected with a clear error
- **PK column** of parent entity is the sample pool for child FK columns
- **FK sampling**: child FK values drawn uniformly with replacement from parent PK column
- **Cardinality enforcement**: see §7 below
- **Per-entity row counts**: global `--rows N` is the default; `--rows-per CUSTOMER=1000,ORDER=5000` overrides per entity. Inconsistencies with cardinality (e.g. `A ||--|| B` but `--rows-per A=10,B=20`) are rejected at startup.

### 5.4 Output

- One file per entity: `<out_dir>/<EntityName>.{csv|parquet}`
- M:N relationships emit an additional junction file: `<out_dir>/<Left>_<Right>.{csv|parquet}` with two FK columns

---

## 6. Supported Mermaid Subset

This is the **complete supported grammar**. Anything outside this table is rejected with a line-numbered error.

### 6.1 File structure

| Construct | Syntax | Notes |
|---|---|---|
| Diagram header | `erDiagram` | Required, must be first non-blank, non-comment line |
| Line comment | `%% any text` | Skipped by scanner |
| Whitespace | spaces, tabs, blank lines | Skipped |

### 6.2 Entity declarations

| Construct | Syntax | Example |
|---|---|---|
| Entity block | `<ENTITY-NAME> { <attributes> }` | `CUSTOMER { int id PK string name }` |
| Entity name | `[A-Z][A-Z0-9_-]*` (uppercase-led) | `CUSTOMER`, `LINE-ITEM`, `ORDER_2024` |
| Attribute | `<type> <name> [<KEY>]` | `int id PK`, `string email` |
| Attribute name | `[a-z][a-zA-Z0-9_]*` | `email`, `customerId` |
| Key marker | `PK` \| `FK` \| `UK` | At most one PK per entity (required); UK accepted but unused in v1 |

### 6.3 Attribute types

| Mermaid type | synthtab generator | Notes |
|---|---|---|
| `int`, `integer`, `bigint` | `INT_INC` | Sequential for PKs |
| `string`, `varchar`, `text` | `STRING` | |
| `uuid` | `UUID` | Recommended for PKs |
| `date` | `DATE` | |
| `datetime`, `timestamp` | `DATE_TIME` | |
| `time` | `TIME` | |
| `decimal`, `float`, `double`, `money` | `PRICE` | |
| `bool`, `boolean` | (Phase 6) | Rejected in v1 with "boolean not yet supported" |

`varchar(255)` and similar parameterized types are stripped to their base name before lookup.

### 6.4 Relationships

| Glyph (left–right) | Meaning | Generator behavior |
|---|---|---|
| `\|\|--\|\|` | exactly-one to exactly-one (1:1) | `count(B) == count(A)`; B.fk_a is bijective |
| `\|\|--o{` | one to zero-or-more (1:N) | Each B.fk_a sampled from A.pk |
| `\|\|--\|{` | one to one-or-more (1:N+) | Same as above + ensure every A.pk appears ≥1× in B |
| `}o--o{` | many-to-many (M:N) | Auto-emit junction table `<A>_<B>(a_id, b_id)` |
| `}\|--\|{` | mandatory many-to-many | Junction table + ensure every A.pk and every B.pk appears ≥1× |
| `}o--\|\|` | many to exactly-one | FK on the "many" side, sampled from PK |
| `o\|--\|\|` | zero-or-one to exactly-one | Same as 1:1 but A may be null on B side |
| `\|\|--o\|` | exactly-one to zero-or-one | 1:1 with B optional |

Syntax: `<EntityA> <leftCard>--<rightCard> <EntityB> : <label>`

- `--` is the identifying-relationship connector. Only `--` is supported.
- `:` label is **required** by Mermaid; we follow that.
- Label content is descriptive only (not used for FK naming).

### 6.5 FK naming convention

For relationship `A ||--o{ B`, the FK column on `B` is determined by:

1. **Explicit:** an attribute on `B` marked `FK` whose name is `<a_lowercase>_id` (e.g. `customer_id` for `CUSTOMER`). **Recommended.**
2. **Implicit:** if no explicit `FK` attribute matches, generator auto-adds a `<a_lowercase>_id` column to `B`.

Multiple relationships between the same two entities must use different FK names; this requires the explicit form (will be a parse error otherwise).

---

## 7. Rejected Syntax (with required error messages)

| Construct | Why rejected | Required error |
|---|---|---|
| Non-identifying relationship `..` | Out of scope for v1 | `line N: non-identifying relationships ('..') not supported; use '--'` |
| Attribute with comment string `string name "the name"` | Defer; parser noise | `line N: attribute comments are not supported in v1; remove the trailing string` |
| `bool` / `boolean` attribute type | Generator support deferred | `line N: type 'boolean' not yet supported (Phase 6); use 'string' as a workaround` |
| Unknown attribute type | Ambiguous mapping | `line N: unknown type 'foo'; supported types: int, string, uuid, date, datetime, decimal, … (see docs/ERD.md §Types)` |
| Entity missing PK | Generator can't FK to it | `entity 'X': no attribute marked PK; every entity must declare exactly one PK` |
| Entity with >1 PK | Composite PKs not modeled | `entity 'X': 2 attributes marked PK ('a', 'b'); composite PKs are not supported in v1` |
| Relationship to undeclared entity | Likely typo | `line N: relationship references undeclared entity 'X'` |
| Cycle in FK graph | Topological sort fails | `cycle detected in FK graph: X → Y → X; v1 requires a DAG` |
| Lowercase entity name | Mermaid convention enforced | `line N: entity name 'customer' must start with an uppercase letter` |
| Relationship glyph not in §6.4 table | Ambiguous cardinality | `line N: unrecognized cardinality '<glyph>'; supported: \|\|, \|o, o\|, }o, o{, }\|, \|{` |
| FK marker on attribute with no matching relationship | Dangling reference | `entity 'X': attribute 'foo' marked FK but no relationship targets entity 'foo'` |
| `--rows-per` contradicting cardinality | Impossible request | `entity 'B' set to 20 rows but relationship 'A \|\|--\|\| B' requires count(B) == count(A) == 10` |
| File missing `erDiagram` header | Not a Mermaid ER file | `expected 'erDiagram' keyword at start of file` |

All errors include the source file path and 1-based line number where applicable, and **exit non-zero**. No partial output is written.

---

## 8. Delivery Milestones

| # | Milestone | Plan | Status |
|---|---|---|---|
| M1 | Port and clean scanner from `feature/erdiagram`; add `synthtab er` subcommand stub | [.claude/plans/er-diagram-generator-m1.plan.md](../plans/er-diagram-generator-m1.plan.md) | done |
| M2 | Parser + `ErdAst` model + validation pass (PK count, undeclared refs) | [.claude/plans/er-diagram-generator-m1.plan.md](../plans/er-diagram-generator-m1.plan.md) | done |
| M3 | Topological generator + 1:1, 1:N, N:1 FK wiring + `MultiFileSink` | [.claude/plans/er-diagram-generator-m1.plan.md](../plans/er-diagram-generator-m1.plan.md) | done |
| M4 | Many-to-many junction table emission | [.claude/plans/er-diagram-generator-m1.plan.md](../plans/er-diagram-generator-m1.plan.md) | done |
| M5 | `docs/ERD.md` with supported syntax reference + README quickstart + ≥80% coverage gate | [.claude/plans/er-diagram-generator-m1.plan.md](../plans/er-diagram-generator-m1.plan.md) | done |

Each milestone follows the TDD workflow (test-first, RED→GREEN→refactor) and lands as its own PR.

---

## 9. Acceptance Criteria

- [ ] `synthtab er tests/fixtures/car_person.mmd --out /tmp/out` writes `CAR.csv`, `PERSON.csv`, `NAMED-DRIVER.csv`
- [ ] Every `NAMED-DRIVER.car_id` exists in `CAR.id`; every `NAMED-DRIVER.person_id` exists in `PERSON.id` (integration test asserts this)
- [ ] M:N fixture produces a junction file with no duplicate `(a_id, b_id)` pairs
- [ ] All errors in §7 are reproducible via fixture files and produce the documented messages
- [ ] `docs/ERD.md` documents the full supported subset (§6) and rejection rules (§7)
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo llvm-cov --fail-under-lines 80` all pass in CI
- [ ] Existing `-s …` flat-table mode behavior is unchanged (regression tests still green)

---

## 10. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Mermaid spec ambiguity | High | Document strict subset (§6); reject anything else (§7) — no silent fallthrough |
| Polars 0.38 multi-file write quirks | Medium | One DataFrame per file via existing `ParquetWriter`/`CsvWriter` traits |
| `--rows-per` contradictions hard to surface | High | Validate at startup before any generation; abort early |
| Implicit FK column collision with user-declared attribute | Medium | Detect; require explicit `FK` marker; clear error |
| Scope creep into seeding / determinism | Medium | Defer `--seed` to Phase 6 explicitly (§3 Non-Goals) |

---

## 11. Open Questions

- Should `UK` (unique key) markers enforce uniqueness in generated values? (v1: parse but ignore; revisit when generators support unique constraints.)
- For M:N junction tables, should row count default to `min(count(A), count(B))`, `max`, or a `--junction-rows` flag? (v1 default: `count(A)`, override via `--rows-per <A>_<B>=N`.)
- Should `--format` accept both per-entity overrides and a default? (v1: single global format only.)

---

## 12. References

- Mermaid ER spec: https://mermaid.js.org/syntax/entityRelationshipDiagram.html
- Current `synthtab` README: `/README.md`
- Current architecture notes: `/CLAUDE.md`
- Abandoned exploration: `feature/erdiagram` (scanner.rs is salvageable, rest is not)
