# pg-diff

> PostgreSQL table data diff tool.

It does a row diffing on 2 tables in separate or same database server by
ordering on all columns and streaming in the result and comparing each row.

**Requirements**

- Both tables must have the columns with the same names

**Missing features**

- No DDL diffing or column type diffing.

## Setup

```
git clone git@github.com:agirorn/pg-diff.git
cd pg-diff
cargo build
```

## Usage

You can run `cargo run -- <ARGS>` or `./target/debug/pg-diff <ARGS>` or just
`pg-diff ARGS` if you copy it to your path

```bash
pg-diff \
  --from-db "postgres://user:pass@localhost:5432/from_db" \
  --to-db "postgres://user:pass@localhost:5432/from_db" \
  --from-table "information_schema.columns" \
  --to-table "information_schema.columns"

Diffing
  From: postgres://user:pass@localhost:5432/from_db information_schema.columns
  To: postgres://user:pass@localhost:5432/from_db information_schema.columns
```


```bash
pg-diff \
  --from-db "postgres://user:pass@localhost:5432/to_db" \
  --to-db "postgres://user:pass@localhost:5432/to_db" \
  --from-table "test_from_table" \
  --to-table "test_to_table"

Diffing
  From: postgres://user:pass@localhost:5432/to_db test_from_table
  To: postgres://user:pass@localhost:5432/to_db test_to_table
"[1,\"v-1\"]"
"[2,\"v-2\"]"
Diffing 3 rows failed for 2 rows.
```
