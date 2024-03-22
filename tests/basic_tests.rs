use anyhow::{bail, Result};
use pg_diff::diff::{diff, Diff, DiffArgs, DiffResult, Different, Same};
use sqlx::Executor;

const FROM_DB: &str = "postgres://user:pass@localhost:5432/from_db";
const TO_DB: &str = "postgres://user:pass@localhost:5432/to_db";

#[tokio::test]
async fn compairing_same_table_from_the_same_database() -> Result<()> {
    let table_name = "t2";
    setup(table_name).await.unwrap();
    let result = diff(DiffArgs {
        from_db: FROM_DB,
        from_table: table_name,
        to_db: FROM_DB,
        to_table: table_name,
    })
    .await;
    teardown(table_name).await.unwrap();
    let result = result.unwrap();

    assert!(matches!(result, DiffResult::Same(Same { .. })));
    if let DiffResult::Same(data) = result {
        assert_eq!(data.rows_affected, 1);
    }

    Ok(())
}

#[tokio::test]
async fn compare_simmilar_tables_2_databases() {
    let table_name = "t1";
    setup(table_name).await.unwrap();
    let result = diff(DiffArgs {
        from_db: FROM_DB,
        from_table: table_name,
        to_db: TO_DB,
        to_table: table_name,
    })
    .await;
    teardown(table_name).await.unwrap();

    let result = result.unwrap();

    assert!(matches!(result, DiffResult::Same(Same { .. })));
    if let DiffResult::Same(data) = result {
        assert_eq!(data.rows_affected, 1);
    }
}

#[tokio::test]
async fn compare_diffrent_tables_2_databases() -> Result<()> {
    let table_name = "t3";
    setup(table_name).await.unwrap();
    insert_into_from(table_name).await.unwrap();
    let result = diff(DiffArgs {
        from_db: FROM_DB,
        from_table: table_name,
        to_db: TO_DB,
        to_table: table_name,
    })
    .await;
    teardown(table_name).await.unwrap();

    let result = result.unwrap();

    if !matches!(result, DiffResult::Different(Different { .. })) {
        bail!("Expected diff to be diffrent: {:#?}", result);
    }
    if let DiffResult::Different(data) = result {
        assert_eq!(data.rows_affected, 2);
        assert_eq!(
            data.diffs,
            vec![Diff {
                from: "[2,\"v-2\"]".to_string(),
                to: "".to_string(),
            }]
        );
    }

    Ok(())
}

async fn setup(table_name: &str) -> Result<()> {
    teardown(table_name).await.unwrap();
    let sql: &str = &format!(
        r#"
        CREATE TABLE {table_name} (
            col_1 INT,
            col_2 VARCHAR(255)
        );
        INSERT INTO {table_name} (col_1, col_2)
        VALUES (1, 'v-1');
        "#
    );
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

async fn insert_into_from(table_name: &str) -> Result<()> {
    let sql: &str = &format!(
        r#"
        INSERT INTO {table_name} (col_1, col_2)
        VALUES (2, 'v-2');
        "#
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(FROM_DB)
        .await
        .unwrap();
    pool.execute(sql).await.unwrap();
    Ok(())
}

async fn teardown(table_name: &str) -> Result<()> {
    let sql: &str = &format!("DROP TABLE IF EXISTS {table_name};");
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
