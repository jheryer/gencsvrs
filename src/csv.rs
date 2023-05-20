#[path = "./output.rs"]
mod output;
use self::output::Output;
use fake::{Fake, Faker};

pub fn create_default_csv(rows: usize, delimiter: char, remove_header: bool) {
    let csv_context = create_default_csv_context(rows, delimiter, remove_header);
    let mut output = output::Console {};
    output_csv(csv_context, &mut output);
}

struct CSVContext {
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
        columns: vec![
            ColumnContext::new(String::from("col0"), value_string),
            ColumnContext::new(String::from("col1"), value_string),
            ColumnContext::new(String::from("col2"), value_string),
            ColumnContext::new(String::from("col3"), value_string),
            ColumnContext::new(String::from("col4"), value_string),
        ],
    }
}

fn random_string() -> String {
    Faker.fake::<String>()
}

fn value_string() -> String {
    String::from("value")
}

/*
T E S T S

 */

#[test]
fn test_create_default_csv() {
    let csv_context = CSVContext {
        rows: 10,
        delimiter: ',',
        remove_header: false,
        columns: vec![
            ColumnContext::new(String::from("col0"), value_string),
            ColumnContext::new(String::from("col1"), value_string),
            ColumnContext::new(String::from("col2"), value_string),
            ColumnContext::new(String::from("col3"), value_string),
        ],
    };

    // let headers: Vec<&str> = csv_context
    //     .columns
    //     .iter()
    //     .map(|col| col.name.as_str())
    //     .collect();

    // let header: String = csv_context
    //     .columns
    //     .iter()
    //     .map(|col| col.name.as_str())
    //     .collect::<Vec<String>>()
    //     .join(",");

    let len = csv_context.columns.len();

    for index in 0..len {
        if let Some(col) = csv_context.columns.get(index) {
            print!("{}", col.name);
            if index != len - 1 {
                print!("{}", csv_context.delimiter);
            }
        }
    }
    print!("\n");

    for _ in 0..csv_context.rows {
        for index in 0..len {
            if let Some(col) = csv_context.columns.get(index) {
                print!("{}", (col.generator)());
                if index != len - 1 {
                    print!("{}", csv_context.delimiter);
                }
            }
        }
        print!("\n");
    }
}
