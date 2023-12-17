use crate::output_types::lib::fake;
use crate::output_types::lib::schema::Schema;
use arrow::array::{ArrayRef, Float32Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field};
use std::sync::Arc;

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
                Arc::new(Int32Array::from(build_data_vector(size, fake::fake_int))) as ArrayRef,
            )),
            "VALUE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::value_string,
                ))) as ArrayRef,
            )),
            "DIGIT" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_digit))) as ArrayRef,
            )),
            "DECIMAL" => columns.push((
                Arc::new(Field::new(element.name, DataType::Float32, false)),
                Arc::new(Float32Array::from(build_data_vector(
                    size,
                    fake::fake_decimal,
                ))) as ArrayRef,
            )),
            "DATE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_date))) as ArrayRef,
            )),
            "TIME" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_time))) as ArrayRef,
            )),
            "DATE_TIME" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_date_time,
                ))) as ArrayRef,
            )),
            "NAME" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_name))) as ArrayRef,
            )),
            "ZIP_CODE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_zipcode,
                ))) as ArrayRef,
            )),
            "STATE_NAME" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_state_name,
                ))) as ArrayRef,
            )),
            "COUNTRY_CODE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_country_code,
                ))) as ArrayRef,
            )),
            "STATE_ABBR" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_state_abbr,
                ))) as ArrayRef,
            )),
            "LAT" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_lat))) as ArrayRef,
            )),
            "LON" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_lon))) as ArrayRef,
            )),
            "PHONE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_phone))) as ArrayRef,
            )),
            "LOREM_WORD" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_lorem_word,
                ))) as ArrayRef,
            )),
            "LOREM_TITLE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_lorem_title,
                ))) as ArrayRef,
            )),
            "LOREM_SENTENCE" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_lorem_sentence,
                ))) as ArrayRef,
            )),
            "LOREM_PARAGRAPH" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(
                    size,
                    fake::fake_lorem_paragraph,
                ))) as ArrayRef,
            )),
            "UUID" => columns.push((
                Arc::new(Field::new(element.name, DataType::Utf8, false)),
                Arc::new(StringArray::from(build_data_vector(size, fake::fake_uuid))) as ArrayRef,
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
        let subject = build_arrow_columns(schema, 10);
        assert_eq!(2, subject.len());
    }
}
