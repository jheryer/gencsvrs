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
    append_target: Option<String>,
    delete_target: Option<String>,
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

        let mut data_frame =
            match create_dataframe(tokenized_schema, rows, append_target, delete_target) {
                Ok(df) => df,
                Err(e) => {
                    eprintln!("Error creating DataFrame: {}", e);
                    return Err(e);
                }
            };

        if csv {
            if file_target.is_some() {
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

        let mut data_frame =
            match create_dataframe(tokenized_schema, rows, append_target, delete_target) {
                Ok(df) => df,
                Err(e) => {
                    eprintln!("Error creating DataFrame: {}", e);
                    return Err(e);
                }
            };

        Console {}.write(&mut data_frame)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parquet_without_file_target_is_rejected() {
        // Previously: silently generated and discarded data, exit 0.
        // Now: must surface a clear error to the user.
        let result = run(
            Some("a:INT,b:STRING".to_string()),
            3,
            None,  // no -f / --file-target
            false, // -c
            true,  // -p
            None,
            None,
        );
        assert!(
            result.is_err(),
            "expected Err when --parquet is set without --file-target"
        );
    }

    #[test]
    fn empty_schema_returns_descriptive_error() {
        let result = run(Some("".to_string()), 3, None, true, false, None, None);
        assert!(result.is_err());
    }
}
