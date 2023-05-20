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
        println!("NOT IMPLEMENTED")
    } else {
        csv::create_default_csv(rows, delimiter, remove_header);
    }

    Ok(())
}
