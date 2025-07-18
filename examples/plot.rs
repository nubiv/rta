use clap::{Arg, Command};
use plotters::prelude::*;
use polars::prelude::*;

use rta::read_from_file;

const OUTPUT_DIR: &str = "charts";

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

    let file_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

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

    let filename = format!("{}/{}.png", OUTPUT_DIR, file_stem);
    println!("Generating plot: {}", filename);
    let root = BitMapBackend::new(&filename, (1200, 600)).into_drawing_area();
    // let root = SVGBackend::new(&filename, (1200, 600)).into_drawing_area();
    root.fill(&WHITE).expect("Failed to fill background");

    let y_min = y.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("LastPrice Timeseries (1 every {} records)", step),
            ("sans-serif", 30),
        )
        .margin(10)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(0..(x.len() - 1), y_min..y_max)
        .expect("Failed to build chart");

    chart
        .configure_mesh()
        .x_labels(10)
        .x_label_formatter(&|idx| {
            if let Some(label) = x.get(*idx) {
                // Split the label into date and time for better readability
                let parts: Vec<&str> = label.split('_').collect();
                if parts.len() == 2 {
                    format!("{}\n{}", parts[0], parts[1])
                } else {
                    label.clone()
                }
            } else {
                "".to_string()
            }
        })
        .x_label_style(
            ("sans-serif", 14)
                .into_font()
                .transform(FontTransform::Rotate90),
        )
        .y_desc("LastPrice")
        .x_desc("ActionDay\nUpdateTime")
        .label_style(("sans-serif", 18).into_font())
        .draw()
        .expect("Failed to draw mesh");

    chart
        .draw_series(LineSeries::new((0..x.len()).map(|i| (i, y[i])), &RED))
        .expect("Failed to draw line series");

    root.present().expect("Unable to write result to file");
    println!("Plot saved successfully to {}", filename);

    Ok(())
}
