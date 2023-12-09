use crate::output_types::lib::fake;
use crate::output_types::lib::schema::Schema;
use arrow::array::{ArrayRef, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field};
use std::sync::Arc;

pub struct ColumnContext {
    pub name: String,
    pub generator: Box<dyn Fn() -> String>,
    pub data_type: DataType,
}

impl ColumnContext {
    fn new(name: String, data_type: DataType, generator: impl Fn() -> String + 'static) -> Self {
        Self {
            name,
            generator: Box::new(generator),
            data_type,
        }
    }
}

fn build_data_vector<T>(size: usize, generator: impl Fn() -> T) -> Vec<T> {
    let mut data: Vec<T> = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(generator());
    }
    data
}

pub fn build_arrow_columns(
    schema: Vec<Schema>,
    size: usize,
) -> Vec<(
    Arc<arrow::datatypes::Field>,
    Arc<(dyn arrow::array::Array + 'static)>,
)> {
    let mut columns: Vec<(
        Arc<arrow::datatypes::Field>,
        Arc<(dyn arrow::array::Array + 'static)>,
    )> = Vec::new();
    for element in schema {
        match element.datatype.as_str() {
            "STRING" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_string,
                ))) as ArrayRef,
            )),
            "INT" => columns.push((
                Arc::new(Field::new(element.name, DataType::Int32, false)),
                Arc::new(Int32Array::from(build_data_vector(
                    size,
                    fake::fake_int_i32,
                ))) as ArrayRef,
            )),
            _ => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::unknown_string,
                ))),
            )),
        }
    }

    return columns;
}

pub fn build_columns(schema: Vec<Schema>) -> Vec<ColumnContext> {
    let mut columns: Vec<ColumnContext> = Vec::new();

    for element in schema {
        match element.datatype.as_str() {
            "STRING" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_string,
            )),
            "INT" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_int,
            )),
            "DIGIT" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_digit,
            )),
            "DECIMAL" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_decimal,
            )),
            "DATE" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_date,
            )),
            "TIME" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_time,
            )),
            "DATE_TIME" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_date_time,
            )),
            "NAME" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_name,
            )),
            "ZIP_CODE" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_zipcode,
            )),
            "COUNTRY_CODE" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_country_code,
            )),
            "STATE_NAME" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_state_name,
            )),
            "STATE_ABBR" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_state_abbr,
            )),
            "LAT" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_lat,
            )),
            "LON" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_lon,
            )),
            "PHONE" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_phone,
            )),
            "LOREM_WORD" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_lorem_word,
            )),
            "LOREM_TITLE" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_lorem_title,
            )),
            "LOREM_SENTENCE" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_lorem_sentence,
            )),
            "LOREM_PARAGRAPH" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_lorem_paragraph,
            )),
            "UUID" => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::fake_uuid,
            )),
            _ => columns.push(ColumnContext::new(
                element.name,
                DataType::Utf8,
                fake::unknown_string,
            )),
        }
    }

    return columns;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_types::lib::schema::Schema;
    #[test]
    fn test_build_columns2() {
        let schema = vec![
            Schema {
                name: String::from("col1"),
                datatype: String::from("STRING"),
            },
            Schema {
                name: String::from("col2"),
                datatype: String::from("INT"),
            },
        ];
        let subject = super::build_arrow_columns(schema, 10);
        assert_eq!(2, subject.len());
        // assert_eq!("col1", subject.get(0).unwrap().name);
        // assert_eq!("STRING", subject.get(0).unwrap().datatype);
        // assert_eq!("col2", subject.get(1).unwrap().name);
        // assert_eq!("INT", subject.get(1).unwrap().datatype);
    }
}
