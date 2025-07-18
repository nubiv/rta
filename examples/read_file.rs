use clap::{Arg, Command};
use polars::prelude::*;

use rta::read_from_file;

fn main() -> PolarsResult<()> {
    let matches = Command::new("read_file")
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .value_name("SOURCE_FILE")
                .help("Sets the source file to use")
                .required(true),
        )
        .get_matches();

    let path = matches
        .get_one::<String>("source")
        .expect("Unable to parse source path...");

    let mut df = read_from_file(path)?;
    // println!("Columns: {:?}", df.get_column_names());

    df.sort_in_place(
        vec!["ActionDay", "UpdateTime"],
        SortMultipleOptions {
            // ascending order for both columns
            descending: vec![false, false],
            // ascending order for both columns
            nulls_last: vec![false, false],
            multithreaded: true,
            maintain_order: false,
            limit: None,
        },
    )?;
    println!("First 5 rows:\n{}", df.head(Some(5)));
    println!("Last 5 rows:\n{}", df.tail(Some(5)));

    Ok(())
}
