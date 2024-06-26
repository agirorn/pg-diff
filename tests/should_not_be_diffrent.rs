use anyhow::Result;
use pg_diff::diff::{diff, DiffArgs, DiffResult, Same};
use sqlx::Executor;

const DB: &str = "postgres://user:pass@localhost:5432/from_db";

#[tokio::test]
async fn compairing_same_table_from_the_same_database() -> Result<()> {
    setup().await.unwrap();
    let result = diff(DiffArgs {
        from_db: DB,
        from_table: "test_from_should_not_be_broken",
        to_db: DB,
        to_table: "test_to_should_not_be_broken",
    })
    .await;
    teardown().await.unwrap();
    let result = result.unwrap();

    assert!(matches!(result, DiffResult::Same(Same { .. })));
    if let DiffResult::Same(data) = result {
        assert_eq!(data.rows_affected, 2);
    }

    Ok(())
}

async fn setup() -> Result<()> {
    teardown().await.unwrap();
    let sql: &str = r#"
        CREATE TABLE test_from_should_not_be_broken (
          col_1 INT,
          col_2 VARCHAR(255)
        );
        CREATE TABLE test_to_should_not_be_broken (
          col_1 INT,
          col_2 VARCHAR(255)
        );
        INSERT INTO test_from_should_not_be_broken (col_1, col_2)
        VALUES (1,'v-1'), (2,'v-2');
        INSERT INTO test_to_should_not_be_broken (col_1, col_2)
        VALUES (2,'v-2'), (1,'v-1');
    "#;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();
    Ok(())
}

async fn teardown() -> Result<()> {
    let sql: &str = r#"
        DROP TABLE IF EXISTS test_from_should_not_be_broken;
        DROP TABLE IF EXISTS test_to_should_not_be_broken;
    "#;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();
    Ok(())
}
