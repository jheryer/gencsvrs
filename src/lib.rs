mod util;
use std::error::Error;
use util::schema::default_schema;
use util::schema::parse_schema;
use util::{dataframe::create_dataframe, output::Console};
use Box;

use crate::util::output::{CSVFile, Output, ParquetFile};
type RunResult<T> = Result<T, Box<dyn Error>>;

pub fn run(
    schema: Option<String>,
    rows: usize,
    file_target: Option<String>,
    csv: bool,
    parquet: bool,
) -> RunResult<()> {
    let csv = if csv == false && parquet == false {
        true
    } else {
        csv
    };

    if let Some(schema) = schema {
        let tokenized_schema = parse_schema(schema.as_str());

        if tokenized_schema.len() == 0 {
            return Err("It has issues.".into());
        }

        let mut data_frame = create_dataframe(tokenized_schema, rows);

        if csv {
            if file_target.is_some() {
                println!("CSV file output");
                CSVFile {
                    file_name: file_target.unwrap(),
                }
                .write(&mut data_frame)?;
            } else {
                Console {}.write(&mut data_frame)?;
            }
        } else if parquet && file_target.is_some() {
            ParquetFile {
                file_name: file_target.unwrap(),
            }
            .write(&mut data_frame)?;
        }
    } else {
        let tokenized_schema = default_schema();
        let mut data_frame = create_dataframe(tokenized_schema, rows);
        Console {}.write(&mut data_frame)?;
    }

    Ok(())
}
