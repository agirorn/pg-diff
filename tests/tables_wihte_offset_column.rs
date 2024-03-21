use anyhow::Result;
use pg_diff::diff::{diff, DiffArgs, DiffResult, Same};
use sqlx::Executor;

const DB: &str = "postgres://user:pass@localhost:5432/from_db";

#[tokio::test]
async fn compairing_same_table_from_the_same_database() -> Result<()> {
    setup().await.unwrap();
    let result = diff(DiffArgs {
        from_db: DB,
        from_table: "from_offset_colum_table",
        to_db: DB,
        to_table: "from_offset_colum_table",
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
        CREATE TABLE from_offset_colum_table (
          col_1 INT,
          "offset" VARCHAR(255)
        );
        INSERT INTO from_offset_colum_table (col_1, "offset")
        VALUES (1,'offset-1'), (2,'offset-2');
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
    let sql: &str = "DROP TABLE IF exists from_offset_colum_table;";
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();
    Ok(())
}
