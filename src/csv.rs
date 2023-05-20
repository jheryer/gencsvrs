#[path = "./fake.rs"]
mod fake;
#[path = "./output.rs"]
mod output;
use self::output::Output;

pub fn create_default_csv(rows: usize, delimiter: char, remove_header: bool) {
    let csv_context = create_default_csv_context(rows, delimiter, remove_header);
    let mut output = output::Console {};
    output_csv(csv_context, &mut output);
}

pub fn parse_schema(input: &str) -> Vec<Schema> {
    let trimmed_input = input.trim_end_matches(',');
    let schema: Vec<Schema> = trimmed_input
        .split(',')
        .filter_map(|column_str| Schema::from_string(column_str))
        .collect();
    return schema;
}

pub fn create_schema_csv_context(
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

struct ColumnContext {
    name: String,
    pub generator: Box<dyn Fn() -> String>,
}

impl ColumnContext {
    fn new(name: String, generator: impl Fn() -> String + 'static) -> Self {
        Self {
            name,
            generator: Box::new(generator),
        }
    }
}

pub struct Schema {
    name: String,
    datatype: String,
}

impl Schema {
    fn from_string(input: &str) -> Option<Schema> {
        let input = input.replace(" ", "");
        let parts: Vec<&str> = input.splitn(2, ':').collect();
        if parts.len() != 2 {
            println!("Bad Schema: {:?} is invalid", parts);
            None
        } else {
            Some(Schema {
                name: parts[0].trim().to_string(),
                datatype: parts[1].trim().to_string(),
            })
        }
    }
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

fn build_columns(schema: Vec<Schema>) -> Vec<ColumnContext> {
    let mut columns: Vec<ColumnContext> = Vec::new();

    for element in schema {
        match element.datatype.as_str() {
            "STRING" => columns.push(ColumnContext::new(element.name, fake::fake_string)),
            "INT" => columns.push(ColumnContext::new(element.name, fake::fake_int)),
            "DECIMAL" => columns.push(ColumnContext::new(element.name, fake::fake_decimal)),
            "DATE" => columns.push(ColumnContext::new(element.name, fake::fake_date)),
            "TIME" => columns.push(ColumnContext::new(element.name, fake::fake_time)),
            "DATE_TIME" => columns.push(ColumnContext::new(element.name, fake::fake_date_time)),
            "NAME" => columns.push(ColumnContext::new(element.name, fake::fake_name)),
            "ZIP_CODE" => columns.push(ColumnContext::new(element.name, fake::fake_zipcode)),
            "COUNTRY_CODE" => {
                columns.push(ColumnContext::new(element.name, fake::fake_country_code))
            }
            "LAT" => columns.push(ColumnContext::new(element.name, fake::fake_lat)),
            "LON" => columns.push(ColumnContext::new(element.name, fake::fake_lon)),
            "PHONE" => columns.push(ColumnContext::new(element.name, fake::fake_phone)),
            "LOREM_WORD" => columns.push(ColumnContext::new(element.name, fake::fake_lorem_word)),
            "LOREM_SENTENCE" => {
                columns.push(ColumnContext::new(element.name, fake::fake_lorem_sentence))
            }
            "LOREM_PARAGRAPH" => {
                columns.push(ColumnContext::new(element.name, fake::fake_lorem_paragraph))
            }
            _ => columns.push(ColumnContext::new(element.name, fake::unknown_string)),
        }
    }

    return columns;
}

fn create_default_csv_context(rows: usize, delimiter: char, remove_header: bool) -> CSVContext {
    CSVContext {
        rows,
        delimiter,
        remove_header,
        columns: vec![
            ColumnContext::new(String::from("col0"), fake::value_string),
            ColumnContext::new(String::from("col1"), fake::value_string),
            ColumnContext::new(String::from("col2"), fake::value_string),
            ColumnContext::new(String::from("col3"), fake::value_string),
            ColumnContext::new(String::from("col4"), fake::value_string),
        ],
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
