# gencsv er — ER Diagram Data Generator

Generate relationally-consistent multi-table CSV/Parquet data from a Mermaid
`erDiagram` source file.

## Quick start

```bash
# Write 100 rows per entity to ./out/
gencsv er schema.mmd -r 100 --out ./out

# Override per-entity row counts
gencsv er schema.mmd -r 100 --rows-per ORDER=5000 --rows-per ITEM=25000

# Parquet output
gencsv er schema.mmd -r 100 --out ./out -F parquet
```

## Supported syntax

### Header

Every file must start with exactly:

```
erDiagram
```

### Entity blocks

```
ENTITY_NAME {
    type  attribute_name  [PK|FK|UK]
}
```

- Entity names: uppercase letters, digits, hyphens (`A-Z0-9-`)
- Attribute names: lowercase letters, digits, underscores (`a-z0-9_`)
- At most one `PK` attribute per entity
- `FK` attributes are auto-wired via relationship edges (see below)
- `UK` is accepted and treated as a regular column for data generation

### Supported Mermaid types

| Mermaid type | gencsv generator |
|---|---|
| `int` | `INT_INC` (auto-increment for PK, `INT` otherwise) |
| `string` | `STRING` |
| `varchar` / `varchar(n)` | `STRING` |
| `boolean` / `bool` | literal `true` / `false` |
| `date` | `DATE` |
| `datetime` | `DATE_TIME` |
| `float` / `double` / `decimal` | `DECIMAL` |
| `uuid` | `UUID` |

Any other type is rejected at parse time with a clear error.

### Relationships

```
LEFT_ENTITY GLYPH RIGHT_ENTITY : "label"
```

| Glyph | Meaning |
|---|---|
| `\|\|--\|\|` | exactly-one : exactly-one |
| `\|\|--o\|` | exactly-one : zero-or-one |
| `o\|--\|\|` | zero-or-one : exactly-one |
| `\|\|--o{` | exactly-one : zero-or-more |
| `\|\|--\|{` | exactly-one : one-or-more |
| `}o--\|\|` | zero-or-more : exactly-one |
| `}o--o{` | zero-or-more : zero-or-more (M:N) |
| `}\|--\|{` | mandatory many-to-many |

Labels are optional. Use `%%` for line comments.

## FK column wiring

For non-M:N relationships the generator adds a FK column to the child entity
automatically:

- If the child entity declares an attribute with `FK` key and a name matching
  `<parent_lowercase>_id`, it is used as-is.
- Otherwise `<parent_lowercase>_id` is appended to the child's column list.

FK values are sampled with replacement from the parent's PK column, ensuring
referential integrity.

## M:N junction tables

A `}o--o{` or `}|--|{` relationship generates a junction table named
`LEFT_RIGHT` (e.g. `STUDENT_COURSE`). The junction table has two FK columns
pointing to each parent's PK. Row count equals `default_rows` (or the
`--rows-per JUNCTION_NAME=N` override).

## Validation rules

The parser rejects the following with a descriptive error and source line number:

- Unknown cardinality glyph (e.g. `||..o{`)
- Unknown Mermaid attribute type
- Duplicate entity names
- More than one `PK` per entity
- Relationship referencing an undeclared entity
- Cyclic FK dependencies (e.g. A → B → A)
- Attribute comment syntax (not supported)

## Example

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

```bash
gencsv er shop.mmd -r 500 --rows-per ORDER=2000 --out ./data
# Writes: data/CUSTOMER.csv  (500 rows)
#         data/ORDER.csv     (2000 rows, customer_id in CUSTOMER.id)
```
