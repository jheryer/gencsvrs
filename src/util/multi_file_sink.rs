//! Per-entity output sink for ER mode. Writes one file per `(name, DataFrame)`
//! pair under a single output directory.

use crate::util::output::{CSVFile, Output, ParquetFile};
use polars::frame::DataFrame;
use std::error::Error;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkFormat {
    Csv,
    Parquet,
}

pub struct MultiFileSink {
    pub out_dir: PathBuf,
    pub format: SinkFormat,
}

impl MultiFileSink {
    pub fn new(out_dir: PathBuf, format: SinkFormat) -> Result<Self, Box<dyn Error>> {
        std::fs::create_dir_all(&out_dir).map_err(|e| {
            format!(
                "failed to create output directory '{}': {e}",
                out_dir.display()
            )
        })?;
        Ok(Self { out_dir, format })
    }

    pub fn write(&self, name: &str, df: &mut DataFrame) -> Result<PathBuf, Box<dyn Error>> {
        let ext = match self.format {
            SinkFormat::Csv => "csv",
            SinkFormat::Parquet => "parquet",
        };
        let path = self.out_dir.join(format!("{name}.{ext}"));
        let path_str = path
            .to_str()
            .ok_or("output path is not valid UTF-8")?
            .to_string();
        match self.format {
            SinkFormat::Csv => CSVFile {
                file_name: path_str,
            }
            .write(df)?,
            SinkFormat::Parquet => ParquetFile {
                file_name: path_str,
            }
            .write(df)?,
        }
        Ok(path)
    }
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;
    use polars::prelude::*;

    fn sample_df() -> DataFrame {
        let s = Series::new("id", vec![1i32, 2, 3]);
        DataFrame::new(vec![s]).unwrap()
    }

    #[test]
    fn creates_directory_and_writes_csv() {
        let tmp = std::env::temp_dir().join("synthtab_msink_csv_test");
        let _ = std::fs::remove_dir_all(&tmp);
        let sink = MultiFileSink::new(tmp.clone(), SinkFormat::Csv).unwrap();
        let mut df = sample_df();
        let path = sink.write("ENTITY", &mut df).unwrap();
        assert!(path.exists());
        assert_eq!(path.extension().and_then(|s| s.to_str()), Some("csv"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn creates_directory_and_writes_parquet() {
        let tmp = std::env::temp_dir().join("synthtab_msink_parquet_test");
        let _ = std::fs::remove_dir_all(&tmp);
        let sink = MultiFileSink::new(tmp.clone(), SinkFormat::Parquet).unwrap();
        let mut df = sample_df();
        let path = sink.write("ENTITY", &mut df).unwrap();
        assert!(path.exists());
        assert_eq!(path.extension().and_then(|s| s.to_str()), Some("parquet"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn file_path_combines_out_dir_and_entity_name() {
        let tmp = std::env::temp_dir().join("synthtab_msink_path_test");
        let _ = std::fs::remove_dir_all(&tmp);
        let sink = MultiFileSink::new(tmp.clone(), SinkFormat::Csv).unwrap();
        let mut df = sample_df();
        let path = sink.write("CUSTOMER", &mut df).unwrap();
        assert_eq!(
            path,
            tmp.join("CUSTOMER.csv"),
            "expected path under out_dir"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
