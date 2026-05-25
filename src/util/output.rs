use std::error::Error;
use std::io::stdout;

use polars::frame::DataFrame;
use polars::prelude::*;

pub trait Output {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>>;
}

pub struct ParquetFile {
    pub file_name: String,
}

impl Output for ParquetFile {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(self.file_name.as_str())
            .map_err(|e| format!("failed to create parquet file '{}': {e}", self.file_name))?;
        ParquetWriter::new(&mut file)
            .finish(df)
            .map_err(|e| format!("failed to write parquet file '{}': {e}", self.file_name))?;
        Ok(())
    }
}

pub struct CSVFile {
    pub file_name: String,
}

impl Output for CSVFile {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(self.file_name.as_str())
            .map_err(|e| format!("failed to create CSV file '{}': {e}", self.file_name))?;
        CsvWriter::new(&mut file)
            .finish(df)
            .map_err(|e| format!("failed to write CSV file '{}': {e}", self.file_name))?;
        Ok(())
    }
}

pub struct Console {}

impl Output for Console {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>> {
        CsvWriter::new(stdout()).finish(df)?;
        Ok(())
    }
}
