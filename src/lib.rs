mod csv;
use std::error::Error;
use Box;
type RunResult<T> = Result<T, Box<dyn Error>>;

pub fn run(
    schema: Option<String>,
    rows: usize,
    delimiter: char,
    remove_header: bool,
) -> RunResult<()> {
    if let Some(schema) = schema {
        println!("NOT IMPLEMENTED");
        let tokenized_schema = csv::parse_schema(schema.as_str());

        if tokenized_schema.len() == 0 {
            return Err("It has issues.".into());
        }

        csv::create_schema_csv_context(tokenized_schema, rows, delimiter, remove_header);
    } else {
        csv::create_default_csv(rows, delimiter, remove_header);
    }

    Ok(())
}
