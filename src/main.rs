pub mod connection_string;
pub mod diff;
use anyhow::Result;
use clap::command;
pub use connection_string::parse_without_user_pass;
pub use diff::{diff, DiffArgs, DiffResult, Different, Same};
use prettydiff::diff_chars;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = command!()
        .arg(
            clap::Arg::new("FROM_DB")
                .short('f')
                .long("from-db")
                .required(true),
        )
        .arg(
            clap::Arg::new("TO_DB")
                .short('t')
                .long("to-db")
                .required(true),
        )
        .arg(
            clap::Arg::new("FROM_TABLE")
                .short('F')
                .long("from-table")
                .required(true),
        )
        .arg(
            clap::Arg::new("TO_TABLE")
                .short('T')
                .long("to-table")
                .required(true),
        )
        .get_matches();

    let from_db: String = matches.get_one::<String>("FROM_DB").unwrap().into();
    let from_table: String = matches.get_one::<String>("FROM_TABLE").unwrap().into();
    let to_db: String = matches.get_one::<String>("TO_DB").unwrap().into();
    let to_table: String = matches.get_one::<String>("TO_TABLE").unwrap().into();

    let from = parse_without_user_pass(&from_db).unwrap();
    let to = parse_without_user_pass(&to_db).unwrap();
    eprintln!("Diffing");
    eprintln!("  From: {from} {from_table}",);
    eprintln!("  To: {to} {to_table}",);

    let result: DiffResult = diff(DiffArgs {
        from_db: &from_db,
        from_table: &from_table,
        to_db: &to_db,
        to_table: &to_table,
    })
    .await
    .unwrap();
    match result {
        DiffResult::Same(data) => {
            print_same(data);
        }
        DiffResult::Different(data) => {
            print_difference(data);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn print_same(data: Same) {
    let Same { rows_affected } = data;
    println!("Diff {rows_affected} rows are all the same");
}

fn print_difference(data: Different) {
    let Different {
        diffs,
        rows_affected,
    } = data;
    let diffrent_lines = diffs.len();
    for row in diffs {
        let from = serde_json::to_string(&row.from).unwrap();
        let to = serde_json::to_string(&row.to).unwrap();
        println!("{}", diff_chars(&from, &to).set_highlight_whitespace(true));
    }
    println!("Diffing {rows_affected} rows failed for {diffrent_lines} rows.",);
}
