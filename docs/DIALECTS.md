# Dialect Support

`synthtab` can emit dialect-correct `CREATE TABLE` DDL and a load-command snippet alongside every generated data file.  Pass `--target <dialect>` to enable this feature.

## Supported Dialects

| Flag value | Database |
|---|---|
| `mysql` | MySQL 8+ |
| `postgres` | PostgreSQL 14+ |
| `sqlserver` | SQL Server 2019+ |
| `bigquery` | Google BigQuery |
| `spark` | Apache Spark / Databricks |

## SQL Type Mapping

| synthtab type | MySQL | Postgres | SQL Server | BigQuery | Spark |
|---|---|---|---|---|---|
| `INT_INC` (PK) | `INT AUTO_INCREMENT PRIMARY KEY` | `SERIAL PRIMARY KEY` | `INT IDENTITY(1,1) PRIMARY KEY` | `INT64` | `BIGINT` |
| `INT_INC` (non-PK) | `INT NOT NULL` | `INTEGER NOT NULL` | `INT NOT NULL` | `INT64` | `BIGINT` |
| `INT` | `INT` | `INTEGER` | `INT` | `INT64` | `BIGINT` |
| `INT_RNG` | `INT` | `INTEGER` | `INT` | `INT64` | `BIGINT` |
| `DIGIT` | `TINYINT` | `SMALLINT` | `TINYINT` | `INT64` | `INT` |
| `DECIMAL` | `DECIMAL(10,2)` | `NUMERIC(10,2)` | `DECIMAL(10,2)` | `NUMERIC` | `DECIMAL(10,2)` |
| `PRICE` | `DECIMAL(10,2)` | `NUMERIC(10,2)` | `DECIMAL(10,2)` | `NUMERIC` | `DECIMAL(10,2)` |
| `STRING` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `VALUE` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `NAME` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `FIRST_NAME` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `LAST_NAME` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `DATE` | `DATE` | `DATE` | `DATE` | `DATE` | `DATE` |
| `TIME` | `TIME` | `TIME` | `TIME` | `TIME` | `STRING` |
| `DATE_TIME` | `DATETIME` | `TIMESTAMP` | `DATETIME2` | `DATETIME` | `TIMESTAMP` |
| `UUID` | `VARCHAR(36)` | `UUID` | `UNIQUEIDENTIFIER` | `STRING` | `STRING` |
| `SSN` | `VARCHAR(11)` | `TEXT` | `NVARCHAR(11)` | `STRING` | `STRING` |
| `ZIP_CODE` | `VARCHAR(10)` | `TEXT` | `NVARCHAR(10)` | `STRING` | `STRING` |
| `COUNTRY_CODE` | `VARCHAR(3)` | `TEXT` | `NVARCHAR(3)` | `STRING` | `STRING` |
| `STATE_NAME` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `STATE_ABBR` | `VARCHAR(2)` | `TEXT` | `NVARCHAR(2)` | `STRING` | `STRING` |
| `LAT` | `DECIMAL(9,6)` | `NUMERIC(9,6)` | `DECIMAL(9,6)` | `FLOAT64` | `DOUBLE` |
| `LON` | `DECIMAL(9,6)` | `NUMERIC(9,6)` | `DECIMAL(9,6)` | `FLOAT64` | `DOUBLE` |
| `PHONE` | `VARCHAR(20)` | `TEXT` | `NVARCHAR(20)` | `STRING` | `STRING` |
| `LOREM_WORD` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `LOREM_TITLE` | `VARCHAR(255)` | `TEXT` | `NVARCHAR(255)` | `STRING` | `STRING` |
| `LOREM_SENTENCE` | `TEXT` | `TEXT` | `NVARCHAR(MAX)` | `STRING` | `STRING` |
| `LOREM_PARAGRAPH` | `TEXT` | `TEXT` | `NVARCHAR(MAX)` | `STRING` | `STRING` |

## Load Commands

Each dialect gets a different load-command file format.

| Dialect | Extension | Command style |
|---|---|---|
| `mysql` | `.sql` | `LOAD DATA LOCAL INFILE` |
| `postgres` | `.sql` | `\copy` |
| `sqlserver` | `.sql` | `BULK INSERT` |
| `bigquery` | `.sh` | `bq load` shell command |
| `spark` | `.py` | PySpark `spark.read` snippet |

### MySQL

```sql
LOAD DATA LOCAL INFILE 'users.csv' INTO TABLE users
FIELDS TERMINATED BY ',' ENCLOSED BY '"'
LINES TERMINATED BY '\n' IGNORE 1 ROWS;
```

### PostgreSQL

```sql
\copy users FROM 'users.csv' WITH (FORMAT csv, HEADER true);
```

### SQL Server

```sql
BULK INSERT users FROM 'users.csv' WITH (FORMAT = 'CSV', FIRSTROW = 2, KEEPIDENTITY);
```

### BigQuery (CSV)

```sh
bq load --source_format=CSV --skip_leading_rows=1 dataset.users users.csv
```

### BigQuery (Parquet)

```sh
bq load --source_format=PARQUET dataset.users users.parquet
```

### Spark (CSV)

```python
spark.read.option("header", True).csv("users.csv").write.saveAsTable("users")
```

### Spark (Parquet)

```python
spark.read.parquet("users.parquet").write.saveAsTable("users")
```

## Parquet Logical Types (BigQuery and Spark)

When using `--format parquet` (ER mode) or `--parquet` (flat mode) with `--target bigquery` or `--target spark`, synthtab emits a warning:

```
warning: --target bigquery with --format parquet: review docs/DIALECTS.md for recommended Parquet logical types
```

This warning exists because Parquet physical types must be annotated with the correct logical type to load cleanly into BigQuery and Spark.

| synthtab type | Recommended Parquet logical type |
|---|---|
| `DATE` | `DATE` (INT32 with DATE annotation) |
| `TIME` | `TIME_MILLIS` or `TIME_MICROS` |
| `DATE_TIME` | `TIMESTAMP_MILLIS` or `TIMESTAMP_MICROS` |
| `DECIMAL` / `PRICE` | `DECIMAL` with `precision=10, scale=2` |
| `LAT` / `LON` | `DOUBLE` |
| `UUID` | `STRING` (UTF8) |

synthtab uses polars to write Parquet, which generally applies correct annotations automatically. Review the Parquet schema with `parquet-tools schema <file>` if you encounter type mismatch errors on load.

## ER Mode DDL

In `synthtab er` mode, a single `schema.ddl.<dialect>.sql` file is written to the output directory. It contains `CREATE TABLE` statements for all entities in dependency order, plus junction tables for many-to-many relationships.

Foreign key constraints are emitted for MySQL, PostgreSQL, and SQL Server. BigQuery and Spark do not enforce FK constraints natively, so they are omitted for those dialects.

```
erDiagram
  CUSTOMER { int id PK }
  ORDER { int id PK }
  CUSTOMER ||--o{ ORDER : places
```

Generates (Postgres):

```sql
CREATE TABLE CUSTOMER (
  id SERIAL PRIMARY KEY
);
CREATE TABLE ORDER (
  id SERIAL PRIMARY KEY,
  customer_id INTEGER,
  CONSTRAINT fk_order_customer_id FOREIGN KEY (customer_id) REFERENCES CUSTOMER(id)
);
```

## Suppressing Output Files

| Flag | Effect |
|---|---|
| `--no-ddl` | Skip DDL file emission |
| `--no-load` | Skip load-command file emission |
