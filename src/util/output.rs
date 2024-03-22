use std::error::Error;
use std::io::stdout;

use polars::frame::DataFrame;
use polars::prelude::*;

pub trait Output {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>>;
}

pub struct MockConsole {
    pub write_was_called: usize,
}
impl Output for MockConsole {
    fn write(&mut self, _df: &mut DataFrame) -> Result<(), Box<dyn Error>> {
        self.write_was_called += 1;
        Ok(())
    }
}

pub struct ParquetFile {
    pub file_name: String,
}

impl Output for ParquetFile {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(self.file_name.as_str())?;
        ParquetWriter::new(&mut file).finish(df).unwrap();
        Ok(())
    }
}

pub struct CSVFile {
    pub file_name: String,
}

impl Output for CSVFile {
    fn write(&mut self, df: &mut DataFrame) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(self.file_name.as_str())?;
        CsvWriter::new(&mut file).finish(df).unwrap();
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
