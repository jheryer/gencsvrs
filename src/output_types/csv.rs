use crate::output_types::lib::arrow_util::create_record_batch;
use crate::output_types::lib::column_context::build_columns;
use crate::output_types::lib::column_context::ColumnContext;
use crate::output_types::lib::output;
use crate::output_types::lib::output::Output;
use crate::output_types::lib::schema::Schema;
pub fn create_default_csv(rows: usize, delimiter: char, remove_header: bool) {
    create_default_csv_context(rows, delimiter, remove_header);
    // let mut output = output::Console {};
    // output_csv(csv_context, &mut output);
}

pub fn create_csv_with_schema(
    schema: Vec<Schema>,
    rows: usize,
    delimiter: char,
    remove_header: bool,
) {
    let csv_context = create_schema_csv_context(schema, rows, delimiter, remove_header);

    let mut output = output::Console {};
    output_csv(csv_context, &mut output);
}

fn create_schema_csv_context(
    schema: Vec<Schema>,
    rows: usize,
    delimiter: char,
    remove_header: bool,
) -> CSVContext {
    let cols = build_columns(schema);
    CSVContext {
        rows,
        delimiter,
        remove_header,
        columns: cols,
    }
}

pub struct CSVContext {
    rows: usize,
    delimiter: char,
    remove_header: bool,
    columns: Vec<ColumnContext>,
}

fn output_csv<T: Output>(csv_context: CSVContext, output: &mut T) {
    let len = csv_context.columns.len();

    if csv_context.remove_header == false {
        for index in 0..len {
            if let Some(col) = csv_context.columns.get(index) {
                output.write(&col.name);
                if index != len - 1 {
                    output.write(&csv_context.delimiter.to_string());
                }
            }
        }
        output.write("\n");
    }

    for _ in 0..csv_context.rows {
        for index in 0..len {
            if let Some(col) = csv_context.columns.get(index) {
                output.write(&(col.generator)());
                if index != len - 1 {
                    output.write(&csv_context.delimiter.to_string());
                }
            }
        }
        output.write("\n");
    }
}

fn create_default_csv_context(rows: usize, delimiter: char, remove_header: bool) {
    let schema = vec![
        Schema {
            name: String::from("col1"),
            datatype: String::from("STRING"),
        },
        Schema {
            name: String::from("col2"),
            datatype: String::from("STRING"),
        },
        Schema {
            name: String::from("col3"),
            datatype: String::from("STRING"),
        },
        Schema {
            name: String::from("col4"),
            datatype: String::from("STRING"),
        },
    ];

    let record = create_record_batch(schema, rows);
    println!("{:?}", record);
}

/*
T E S T S

 */

// #[test]
// fn test_create_default_csv() {
//     let subject = create_default_csv_context(5, ',', false);
//     assert_eq!(5, subject.columns.len());
//     assert_eq!("col0", subject.columns.get(0).unwrap().name);
//     assert_eq!("col1", subject.columns.get(1).unwrap().name);
//     assert_eq!("col2", subject.columns.get(2).unwrap().name);
//     assert_eq!("col3", subject.columns.get(3).unwrap().name);
//     assert_eq!("col4", subject.columns.get(4).unwrap().name);
// }

// #[test]
// fn test_output_with_default_csv_context() {
//     let subject = create_default_csv_context(5, ',', false);
//     let mut output = output::MockConsole {
//         write_was_called: 0,
//     };

//     output_csv(subject, &mut output);
//     assert_eq!(60, output.write_was_called);
// }
