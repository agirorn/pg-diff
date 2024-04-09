use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;
use sqlx::Executor;
use sqlx_pgrow_serde::read_row;

pub struct DiffArgs<'a> {
    pub from_db: &'a str,
    pub from_table: &'a str,
    pub to_db: &'a str,
    pub to_table: &'a str,
}

#[derive(Debug)]
struct ColumnName {
    column_name: Option<String>,
}

pub async fn diff(args: DiffArgs<'_>) -> Result<DiffResult> {
    let DiffArgs {
        from_db,
        from_table,
        to_db,
        to_table,
    } = args;
    let from_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(from_db)
        .await
        .unwrap();
    let to_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(to_db)
        .await
        .unwrap();
    let res: Vec<&str> = from_table.split('.').collect();
    let mut table = from_table;
    let mut schema = "public";

    if res.len() != 1 {
        schema = res[0];
        table = res[1];
    }

    let column_names: Vec<ColumnName> = sqlx::query_as!(
        ColumnName,
        r#"
            SELECT column_name
            FROM information_schema.columns
            WHERE table_schema = $1
            AND table_name = $2
        "#,
        schema,
        table,
    )
    .fetch_all(&from_pool)
    .await
    .unwrap();

    let column_names = column_names
        .into_iter()
        .map(|r: ColumnName| r.column_name)
        .filter(|r| r.is_some())
        .map(|r| match r {
            Some(v) => format!("\"{v}\""),
            None => "".to_string(),
        })
        .collect::<Vec<String>>();

    let order_column_names = column_names.clone();
    let order_column_names = order_column_names
        .into_iter()
        .map(|l| format!("{}::TEXT", l))
        .collect::<Vec<String>>()
        .join(", ");
    let column_names = column_names
        .clone()
        .into_iter()
        .map(|l| format!("{}::TEXT", l))
        .collect::<Vec<String>>()
        .join(", ");
    let order_by = format!("ORDER BY {order_column_names} ASC");

    let sql: &str = &format!("select {column_names} from {from_table} {order_by}",);
    let mut from_rows = from_pool.fetch_many(sql);
    let sql: &str = &format!("select {column_names} from {to_table} {order_by}",);
    let mut to_rows = to_pool.fetch_many(sql);
    let mut rows_affected = 0;
    let mut diffs: Vec<Diff> = vec![];
    while let Some(from_row) = from_rows.next().await {
        let from_row = from_row.unwrap();
        match from_row {
            sqlx::Either::Right(from_row) => {
                let from_row = read_row(&from_row);
                let to_row = to_rows.next().await;
                match to_row {
                    None => {
                        diffs.push(Diff::new_empty_from(&from_row).unwrap());
                    }
                    Some(to_row) => {
                        let to_row = to_row.unwrap();

                        match to_row {
                            sqlx::Either::Right(to_row) => {
                                let to_row = read_row(&to_row);
                                if from_row != to_row {
                                    diffs.push(Diff::new(&from_row, &to_row).unwrap());
                                }
                            }
                            sqlx::Either::Left(_) => {
                                diffs.push(Diff::new_empty_from(&from_row).unwrap());
                            }
                        }
                    }
                }
            }
            sqlx::Either::Left(result) => rows_affected = result.rows_affected(),
        }
    }

    if !diffs.is_empty() {
        return Ok(DiffResult::Different(Different {
            rows_affected,
            diffs,
        }));
    }

    Ok(DiffResult::Same(Same { rows_affected }))
}

fn json_string(v: &Vec<Value>) -> Result<String> {
    Ok(serde_json::to_string(v).unwrap())
}

#[derive(Debug)]
pub struct Same {
    pub rows_affected: u64,
}

#[derive(Debug, PartialEq)]
pub struct Diff {
    pub from: String,
    pub to: String,
}

impl Diff {
    pub fn new(from: &Vec<Value>, to: &Vec<Value>) -> Result<Self> {
        let from = json_string(from).unwrap();
        let to = json_string(to).unwrap();
        Ok(Self { from, to })
    }

    pub fn new_empty_from(v: &Vec<Value>) -> Result<Self> {
        Ok(Self {
            from: json_string(v).unwrap(),
            to: "".to_string(),
        })
    }
}

#[derive(Debug)]
pub struct Different {
    pub rows_affected: u64,
    pub diffs: Vec<Diff>,
}

#[derive(Debug)]
pub enum DiffResult {
    Same(Same),
    Different(Different),
}
