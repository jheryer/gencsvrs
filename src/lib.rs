mod output_types;
use output_types::csv;
use output_types::lib::schema::parse_schema;
use std::error::Error;
use Box;

type RunResult<T> = Result<T, Box<dyn Error>>;

pub fn run(schema: Option<String>, rows: usize) -> RunResult<()> {
    if let Some(schema) = schema {
        let tokenized_schema = parse_schema(schema.as_str());

        if tokenized_schema.len() == 0 {
            return Err("It has issues.".into());
        }

        csv::create_csv_with_schema(tokenized_schema, rows);
    } else {
        csv::create_default_csv(rows);
    }

    Ok(())
}
