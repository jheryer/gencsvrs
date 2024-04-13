use crate::util::fake;
use crate::util::schema::Schema;
use polars::prelude::*;

pub fn create_dataframe(schema: Vec<Schema>, size: usize) -> DataFrame {
    let mut cols = Vec::new();

    for element in schema {
        let col = match element.datatype.as_str() {
            "STRING" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_string),
            ),
            "INT" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_int),
            ),
            "INT_INC" => Series::new(element.name.as_str(), build_incremental_int(size)),
            "VALUE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::value_string),
            ),
            "DIGIT" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_digit),
            ),
            "DECIMAL" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_decimal),
            ),
            "DATE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_date),
            ),
            "TIME" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_time),
            ),
            "DATE_TIME" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_date_time),
            ),
            "NAME" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_name),
            ),
            "ZIP_CODE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_zipcode),
            ),
            "COUNTRY_CODE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_country_code),
            ),
            "STATE_NAME" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_state_name),
            ),
            "STATE_ABBR" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_state_abbr),
            ),
            "LAT" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_lat),
            ),
            "LON" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_lon),
            ),
            "PHONE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_phone),
            ),
            "LOREM_WORD" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_lorem_word),
            ),
            "LOREM_TITLE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_lorem_title),
            ),
            "LOREM_SENTENCE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_lorem_sentence),
            ),
            "LOREM_PARAGRAPH" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_lorem_paragraph),
            ),
            "UUID" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_uuid),
            ),
            _ => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::unknown_string),
            ),
        };

        cols.push(col);
    }

    DataFrame::new(cols).unwrap()
}

fn build_data_vector<T>(size: usize, generator: impl Fn() -> T) -> Vec<T> {
    let mut data: Vec<T> = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(generator());
    }
    data
}

fn build_incremental_int(size: usize) -> Vec<i32> {
    (0..size as i32).collect::<Vec<i32>>()
}
