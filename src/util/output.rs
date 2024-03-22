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

// fn csv_writer<W: io::Write>(
//     mut writer: Writer<W>,
//     record: &RecordBatch,
// ) -> Result<(), Box<dyn Error>> {
//     let cols = record.columns();
//     let record_schema = record.schema();
//     let num_rows = record.num_rows();
//     let num_cols = record.num_columns();
//     let list = record_schema
//         .fields()
//         .iter()
//         .map(|field| field.name().to_string())
//         .collect::<Vec<_>>();

//     writer.write_record(&list)?;

//     for row_index in 0..num_rows {
//         let mut row: Vec<String> = Vec::new();
//         for col_index in 0..num_cols {
//             let col = cols.get(col_index).unwrap();
//             let value = col
//                 .as_any()
//                 .downcast_ref::<arrow::array::StringArray>()
//                 .unwrap()
//                 .value(row_index);
//             row.push(value.to_string());
//         }
//         writer.write_record(&row)?;
//     }
//     writer.flush()?;

//     Ok(())
// }
