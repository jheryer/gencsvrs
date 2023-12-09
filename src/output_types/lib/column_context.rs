use crate::output_types::lib::fake;
use crate::output_types::lib::schema::Schema;

pub struct ColumnContext {
    pub name: String,
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

pub fn default_columns() -> Vec<ColumnContext> {
    vec![
        ColumnContext::new(String::from("col0"), fake::value_string),
        ColumnContext::new(String::from("col1"), fake::value_string),
        ColumnContext::new(String::from("col2"), fake::value_string),
        ColumnContext::new(String::from("col3"), fake::value_string),
        ColumnContext::new(String::from("col4"), fake::value_string),
    ]
}

pub fn build_columns(schema: Vec<Schema>) -> Vec<ColumnContext> {
    let mut columns: Vec<ColumnContext> = Vec::new();

    for element in schema {
        match element.datatype.as_str() {
            "STRING" => columns.push(ColumnContext::new(element.name, fake::fake_string)),
            "INT" => columns.push(ColumnContext::new(element.name, fake::fake_int)),
            "DIGIT" => columns.push(ColumnContext::new(element.name, fake::fake_digit)),
            "DECIMAL" => columns.push(ColumnContext::new(element.name, fake::fake_decimal)),
            "DATE" => columns.push(ColumnContext::new(element.name, fake::fake_date)),
            "TIME" => columns.push(ColumnContext::new(element.name, fake::fake_time)),
            "DATE_TIME" => columns.push(ColumnContext::new(element.name, fake::fake_date_time)),
            "NAME" => columns.push(ColumnContext::new(element.name, fake::fake_name)),
            "ZIP_CODE" => columns.push(ColumnContext::new(element.name, fake::fake_zipcode)),
            "COUNTRY_CODE" => {
                columns.push(ColumnContext::new(element.name, fake::fake_country_code))
            }
            "STATE_NAME" => columns.push(ColumnContext::new(element.name, fake::fake_state_name)),
            "STATE_ABBR" => columns.push(ColumnContext::new(element.name, fake::fake_state_abbr)),
            "LAT" => columns.push(ColumnContext::new(element.name, fake::fake_lat)),
            "LON" => columns.push(ColumnContext::new(element.name, fake::fake_lon)),
            "PHONE" => columns.push(ColumnContext::new(element.name, fake::fake_phone)),
            "LOREM_WORD" => columns.push(ColumnContext::new(element.name, fake::fake_lorem_word)),
            "LOREM_TITLE" => columns.push(ColumnContext::new(element.name, fake::fake_lorem_title)),
            "LOREM_SENTENCE" => {
                columns.push(ColumnContext::new(element.name, fake::fake_lorem_sentence))
            }
            "LOREM_PARAGRAPH" => {
                columns.push(ColumnContext::new(element.name, fake::fake_lorem_paragraph))
            }
            "UUID" => columns.push(ColumnContext::new(element.name, fake::fake_uuid)),
            _ => columns.push(ColumnContext::new(element.name, fake::unknown_string)),
        }
    }

    return columns;
}
