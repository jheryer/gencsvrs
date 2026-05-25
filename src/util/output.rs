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

#[cfg(test)]
mod test {
    use super::*;

    fn sample_df() -> DataFrame {
        let s1 = Series::new("id", vec![0i32, 1, 2]);
        let s2 = Series::new("val", vec!["a", "b", "c"]);
        DataFrame::new(vec![s1, s2]).unwrap()
    }

    #[test]
    fn test_csv_file_writer_creates_file() {
        let path = std::env::temp_dir().join("gencsv_test_csv_writer.csv");
        let mut writer = CSVFile {
            file_name: path.to_str().unwrap().to_string(),
        };
        let mut df = sample_df();
        writer.write(&mut df).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("id"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_parquet_file_writer_creates_file() {
        let path = std::env::temp_dir().join("gencsv_test_parquet_writer.parquet");
        let mut writer = ParquetFile {
            file_name: path.to_str().unwrap().to_string(),
        };
        let mut df = sample_df();
        writer.write(&mut df).unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_csv_file_writer_bad_path_returns_error() {
        let mut writer = CSVFile {
            file_name: "/nonexistent/dir/out.csv".to_string(),
        };
        let mut df = sample_df();
        assert!(writer.write(&mut df).is_err());
    }

    #[test]
    fn test_parquet_file_writer_bad_path_returns_error() {
        let mut writer = ParquetFile {
            file_name: "/nonexistent/dir/out.parquet".to_string(),
        };
        let mut df = sample_df();
        assert!(writer.write(&mut df).is_err());
    }
}
