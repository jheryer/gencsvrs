use crate::output_types::lib::arrow_util::create_record_batch;
use crate::output_types::lib::output;
use crate::output_types::lib::output::Output;
use crate::output_types::lib::schema::Schema;

pub fn create_csv_with_schema(schema: Vec<Schema>, rows: usize) {
    let record = create_record_batch(schema, rows);
    let mut writer = output::Console {};

    if let Err(e) = writer.write(&record) {
        println!("Error: {}", e);
    }
}

pub fn create_default_csv(rows: usize) {
    let schema = vec![
        Schema {
            name: String::from("col1"),
            datatype: String::from("VALUE"),
        },
        Schema {
            name: String::from("col2"),
            datatype: String::from("VALUE"),
        },
        Schema {
            name: String::from("col3"),
            datatype: String::from("VALUE"),
        },
        Schema {
            name: String::from("col4"),
            datatype: String::from("VALUE"),
        },
    ];

    create_csv_with_schema(schema, rows);
}

/*
T E S T S

 */
#[cfg(test)]
mod test {}
