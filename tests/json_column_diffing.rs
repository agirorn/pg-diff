use anyhow::Result;
use pg_diff::diff::{diff, DiffArgs, DiffResult, Same};
use sqlx::Executor;

const FROM_DB: &str = "postgres://user:pass@localhost:5432/from_db";
const TO_DB: &str = "postgres://user:pass@localhost:5432/to_db";

#[tokio::test]
async fn compare_tables_with_json_or_jsonb_columns() -> Result<()> {
    let json_table = "json_table";
    setup().await.unwrap();
    let result = diff(DiffArgs {
        from_db: FROM_DB,
        from_table: json_table,
        to_db: FROM_DB,
        to_table: json_table,
    })
    .await;
    let result = result.unwrap();

    assert!(matches!(result, DiffResult::Same(Same { .. })));
    if let DiffResult::Same(data) = result {
        assert_eq!(data.rows_affected, 2);
    }

    let jsonb_table = "jsonb_table";
    let result = diff(DiffArgs {
        from_db: FROM_DB,
        from_table: jsonb_table,
        to_db: FROM_DB,
        to_table: jsonb_table,
    })
    .await;
    let result = result.unwrap();

    assert!(matches!(result, DiffResult::Same(Same { .. })));
    if let DiffResult::Same(data) = result {
        assert_eq!(data.rows_affected, 2);
    }

    teardown().await.unwrap();

    Ok(())
}

async fn setup() -> Result<()> {
    teardown().await.unwrap();
    let sql: &str = r#"
            CREATE TABLE jsonb_table (col_json JSONB, col_1 INT);
            INSERT INTO jsonb_table (col_json, col_1)
            VALUES ('{"key":"JSON_B_VALUE-2"}', 2)
                 , ('{"key":"JSON_B_VALUE-1"}', 1);

            CREATE TABLE json_table (col_json JSON, col_1 INT);
            INSERT INTO json_table (col_json, col_1)
            VALUES ('{"key":"JSON-VALUE-2"}', 2)
                 , ('{"key":"JSON-VALUE-1"}', 1);
        "#;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(FROM_DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();
    Ok(())
}

async fn teardown() -> Result<()> {
    let sql: &str = r#"
        DROP TABLE IF EXISTS json_table;
        DROP TABLE IF EXISTS jsonb_table;
    "#;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(FROM_DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(TO_DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();
    Ok(())
}
