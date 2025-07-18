use std::path::Path;

use polars::prelude::*;

pub fn read_from_file<P: AsRef<Path>>(path: P) -> PolarsResult<DataFrame> {
    let path = path.as_ref();
    match path.extension().and_then(|s| s.to_str()) {
        Some("csv") => CsvReader::new(std::fs::File::open(path)?).finish(),
        Some("parquet") => ParquetReader::new(std::fs::File::open(path)?).finish(),
        _ => Err(PolarsError::ComputeError(
            "Unsupported file extension. Use .csv or .parquet.".into(),
        )),
    }
}
