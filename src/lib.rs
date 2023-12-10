mod output_types;
use output_types::csv;
use output_types::lib::output::CSVFile;
use output_types::lib::output::Console;
use output_types::lib::schema::parse_schema;
use std::error::Error;
use Box;
type RunResult<T> = Result<T, Box<dyn Error>>;

pub fn run(
    schema: Option<String>,
    rows: usize,
    file_target: Option<String>,
    csv: bool,
    parquet: bool,
) -> RunResult<()> {
    if let Some(schema) = schema {
        let tokenized_schema = parse_schema(schema.as_str());

        if tokenized_schema.len() == 0 {
            return Err("It has issues.".into());
        }
        if csv {
            if file_target.is_some() {
                csv::create_csv_with_schema(
                    tokenized_schema,
                    rows,
                    &mut CSVFile {
                        file_name: file_target.unwrap(),
                    },
                );
            } else {
                csv::create_csv_with_schema(tokenized_schema, rows, &mut Console {});
            }
        } else if parquet && file_target.is_some() {
            println!("Parquet output");
        }
    } else {
        csv::create_default_csv(rows);
    }

    Ok(())
}
