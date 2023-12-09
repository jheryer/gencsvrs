use crate::output_types::lib::column_context::build_columns;
use crate::output_types::lib::column_context::default_columns;
use crate::output_types::lib::column_context::ColumnContext;
use crate::output_types::lib::fake;
use crate::output_types::lib::output;
use crate::output_types::lib::output::Output;
use crate::output_types::lib::schema::Schema;

pub fn create_default_csv(rows: usize, delimiter: char, remove_header: bool) {
    let csv_context = create_default_csv_context(rows, delimiter, remove_header);
    let mut output = output::Console {};
    output_csv(csv_context, &mut output);
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

fn create_default_csv_context(rows: usize, delimiter: char, remove_header: bool) -> CSVContext {
    CSVContext {
        rows,
        delimiter,
        remove_header,
        columns: default_columns(),
    }
}

/*
T E S T S

 */

#[test]
fn test_create_default_csv() {
    let subject = create_default_csv_context(5, ',', false);
    assert_eq!(5, subject.columns.len());
    assert_eq!("col0", subject.columns.get(0).unwrap().name);
    assert_eq!("col1", subject.columns.get(1).unwrap().name);
    assert_eq!("col2", subject.columns.get(2).unwrap().name);
    assert_eq!("col3", subject.columns.get(3).unwrap().name);
    assert_eq!("col4", subject.columns.get(4).unwrap().name);
}

#[test]
fn test_output_with_default_csv_context() {
    let subject = create_default_csv_context(5, ',', false);
    let mut output = output::MockConsole {
        write_was_called: 0,
    };

    output_csv(subject, &mut output);
    assert_eq!(60, output.write_was_called);
}

#[test]
fn test_happy_path_schema_parser() {
    let input = "col1:STRING, col2:INT, col3:DATE, ";
    let subject = parse_schema(input);

    assert_eq!(3, subject.len());
    assert_eq!("col1", subject.get(0).unwrap().name);
    assert_eq!("STRING", subject.get(0).unwrap().datatype);
    assert_eq!("col2", subject.get(1).unwrap().name);
    assert_eq!("INT", subject.get(1).unwrap().datatype);
    assert_eq!("col3", subject.get(2).unwrap().name);
    assert_eq!("DATE", subject.get(2).unwrap().datatype);
}

#[test]
fn test_empty_schema_has_no_results() {
    let input = "";
    let subject = parse_schema(input);
    assert_eq!(0, subject.len());
}

#[test]
fn test_bad_schema_has_no_results() {
    let input = "naughtyschema,,23234kj23lk4j232lkjc 2lkj3 ";
    let subject = parse_schema(input);
    assert_eq!(0, subject.len());
}
