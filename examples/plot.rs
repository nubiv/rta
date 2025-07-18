use clap::{Arg, Command};
use plotly::{Plot, Scatter};
use polars::prelude::*;

use rta::read_from_file;

// const OUTPUT_DIR: &str = "charts";

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
    println!("Reading data from file: {}", path);

    // let file_stem = std::path::Path::new(path)
    //     .file_stem()
    //     .and_then(|s| s.to_str())
    //     .unwrap_or("output");

    let mut df = read_from_file(path)?;
    println!("Data loaded. Number of rows: {}", df.height());
    // println!("Columns: {:?}", df.get_column_names());

    println!("Sorting data by ActionDay and UpdateTime...");
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

    // Compose x-axis from "ActionDay" and "UpdateTime", y-axis from "LastPrice"
    let action_day = df
        .column("ActionDay")?
        .str()
        .expect("ActionDay is not Utf8 type");
    let update_time = df
        .column("UpdateTime")?
        .str()
        .expect("UpdateTime is not Utf8 type");
    let last_price = df
        .column("LastPrice")?
        .f64()
        .expect("LastPrice is not f64 type");

    let n = df.height();

    // Pick one every 120 records
    let step = 120;
    let mut x: Vec<String> = Vec::new();
    let mut y: Vec<f64> = Vec::new();

    println!("Sampling every {} records for plotting...", step);
    for i in (0..n).step_by(step) {
        let day = action_day.get(i).unwrap_or("");
        let time = update_time.get(i).unwrap_or("");
        x.push(format!("{}_{}", day, time));
        y.push(last_price.get(i).unwrap_or(f64::NAN));
    }
    println!("Sampled {} points for the plot.", x.len());

    println!("Generating plotly plot...");
    // Prepare x-axis labels (ActionDay_UpdateTime) and y-axis (LastPrice)
    let trace = Scatter::new(x.clone(), y.clone())
        .mode(plotly::common::Mode::Lines)
        .name("LastPrice");

    let mut plot = Plot::new();
    plot.add_trace(trace);
    plot.set_layout(
        plotly::Layout::new()
            .title(format!("LastPrice Timeseries (1 every {} records)", step))
            .x_axis(
                plotly::layout::Axis::new()
                    .title("ActionDay_UpdateTime")
                    .tick_angle(45.0)
                    .auto_margin(true),
            )
            .y_axis(plotly::layout::Axis::new().title("LastPrice")),
    );

    plot.show();
    println!("Plot displayed in browser...");

    Ok(())
}
