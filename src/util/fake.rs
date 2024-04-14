extern crate fakeit;
use crate::util::schema::Schema;
use fake::faker::address::raw::*;
use fake::faker::chrono::raw::*;
use fake::faker::lorem::raw::*;
use fake::faker::name::raw::*;
use fake::faker::number::raw::*;
use fake::faker::phone_number::raw::*;
use fake::locales::*;
use fake::{Fake, Faker};
use fakeit::currency;
use fakeit::name;
use fakeit::person;
use polars::prelude::*;
use regex::Regex;
use std::error::Error;
use uuid::Uuid;

type RangeParseResult = Result<(i32, i32), Box<dyn Error>>;

fn build_data_vector<T>(size: usize, generator: impl Fn() -> T) -> Vec<T> {
    let mut data: Vec<T> = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(generator());
    }
    data
}

pub fn build_incremental_int(size: i32, start: i32, end: i32) -> Vec<i32> {
    let end = if start - end < 0 { start + size } else { end };
    (start..end as i32).collect::<Vec<i32>>()
}

fn parse_range_string(range_str: &str) -> RangeParseResult {
    let re = Regex::new(r"\((-?\d+)\s*-\s*(-?\d+)\)").unwrap();
    if let Some(caps) = re.captures(range_str) {
        let lower: i32 = caps.get(1).unwrap().as_str().parse()?;
        let upper: i32 = caps.get(2).unwrap().as_str().parse()?;
        Ok((lower, upper))
    } else {
        Err(format!("Error parsing {}", range_str).into())
    }
}

pub fn create_column(element: Schema, size: usize) -> Series {
    let col = match element.datatype.as_str() {
        "STRING" => Series::new(element.name.as_str(), build_data_vector(size, fake_string)),
        "INT" => Series::new(element.name.as_str(), build_data_vector(size, fake_int)),
        "INT_INC" => Series::new(
            element.name.as_str(),
            build_incremental_int(size as i32, 0, size.clone() as i32),
        ),
        "INT_RNG" => {
            let (lower, upper) =
                match parse_range_string(element.modifier.as_ref().unwrap().as_str()) {
                    Ok((lower, upper)) => (lower, upper),
                    Err(e) => {
                        eprintln!("Error parsing range: {} , using default range", e);
                        (0, size as i32)
                    }
                };

            Series::new(
                element.name.as_str(),
                build_incremental_int(size as i32, lower, upper),
            )
        }
        "VALUE" => Series::new(element.name.as_str(), build_data_vector(size, value_string)),
        "DIGIT" => Series::new(element.name.as_str(), build_data_vector(size, fake_digit)),
        "DECIMAL" => Series::new(element.name.as_str(), build_data_vector(size, fake_decimal)),
        "DATE" => Series::new(element.name.as_str(), build_data_vector(size, fake_date)),
        "TIME" => Series::new(element.name.as_str(), build_data_vector(size, fake_time)),
        "DATE_TIME" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_date_time),
        ),
        "NAME" => Series::new(element.name.as_str(), build_data_vector(size, fake_name)),
        "ZIP_CODE" => Series::new(element.name.as_str(), build_data_vector(size, fake_zipcode)),
        "COUNTRY_CODE" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_country_code),
        ),
        "STATE_NAME" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_state_name),
        ),
        "STATE_ABBR" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_state_abbr),
        ),
        "LAT" => Series::new(element.name.as_str(), build_data_vector(size, fake_lat)),
        "LON" => Series::new(element.name.as_str(), build_data_vector(size, fake_lon)),
        "PHONE" => Series::new(element.name.as_str(), build_data_vector(size, fake_phone)),
        "PRICE" => Series::new(element.name.as_str(), build_data_vector(size, fake_price)),
        "LOREM_WORD" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_lorem_word),
        ),
        "LOREM_TITLE" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_lorem_title),
        ),
        "LOREM_SENTENCE" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_lorem_sentence),
        ),
        "LOREM_PARAGRAPH" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_lorem_paragraph),
        ),
        "UUID" => Series::new(element.name.as_str(), build_data_vector(size, fake_uuid)),
        "FIRST_NAME" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_first_name),
        ),
        "LAST_NAME" => Series::new(
            element.name.as_str(),
            build_data_vector(size, fake_last_name),
        ),
        "SSN" => Series::new(element.name.as_str(), build_data_vector(size, fake_ssn)),
        _ => Series::new(
            element.name.as_str(),
            build_data_vector(size, unknown_string),
        ),
    };
    col
}

//STRING
pub fn fake_string() -> String {
    Faker.fake::<String>()
}
//INT
//Digit

pub fn fake_int() -> i32 {
    return (0..2147483647).fake::<i32>();
}

pub fn fake_digit() -> String {
    Digit(EN).fake()
}
// DECIMAL
pub fn fake_decimal() -> f32 {
    return (0.0..100000.0).fake::<f32>();
}
//DATE
pub fn fake_date() -> String {
    Date(EN).fake()
}
//TIME
pub fn fake_time() -> String {
    Time(EN).fake()
}

//DATE_TIME
pub fn fake_date_time() -> String {
    DateTime(EN).fake()
}
//NAME
pub fn fake_name() -> String {
    Name(EN).fake()
}

//ZIP_CODE
pub fn fake_zipcode() -> String {
    PostCode(EN).fake()
}
//COUNTRY_CODE
pub fn fake_country_code() -> String {
    CountryCode(EN).fake()
}
//STATE_NAME
pub fn fake_state_name() -> String {
    StateName(EN).fake()
}
//STATE_ABBR
pub fn fake_state_abbr() -> String {
    StateAbbr(EN).fake()
}
//LAT
pub fn fake_lat() -> String {
    Latitude(EN).fake()
}
//LON
pub fn fake_lon() -> String {
    Longitude(EN).fake()
}
//PHONE
pub fn fake_phone() -> String {
    CellNumber(EN).fake()
}

//LOREM_WORD
pub fn fake_lorem_word() -> String {
    Word(EN).fake()
}

//LOREM_TITLE
pub fn fake_lorem_title() -> String {
    let title: Vec<String> = Words(EN, 1..4).fake();
    let cap_title: Vec<String> = title
        .iter()
        .map(|s| s[0..1].to_uppercase() + &s[1..])
        .collect();
    cap_title.join(" ")
}

//LOREM_SENTENCE
pub fn fake_lorem_sentence() -> String {
    Sentence(EN, 1..15).fake()
}

//LOREM_PARAGRAPH
pub fn fake_lorem_paragraph() -> String {
    Paragraph(EN, 1..100).fake()
}

//UUID
pub fn fake_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

//FIRST_NAME
pub fn fake_first_name() -> String {
    name::first()
}
//LAST_NAME
pub fn fake_last_name() -> String {
    name::last()
}

//SSN
pub fn fake_ssn() -> String {
    person::ssn()
}

//PRICE
pub fn fake_price() -> String {
    currency::price(0.0, 9999.0).to_string()
}

//default
pub fn value_string() -> String {
    String::from("value")
}
pub fn unknown_string() -> String {
    String::from("unknown")
}
mod test {
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn test_build_incremental_int() {
        let data = build_incremental_int(10, 0, 10);
        assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
    #[test]
    fn test_build_incremental_int_with_negative() {
        let data = build_incremental_int(10, -10, 10);
        assert_eq!(data, vec![-10, -9, -8, -7, -6, -5, -4, -3, -2, -1]);
    }

    #[test]
    fn test_build_incremental_underun_size() {
        let data = build_incremental_int(10, 0, 5);
        assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
    #[test]
    fn test_build_incremental_overrun_size() {
        let data = build_incremental_int(10, 0, 200);
        assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_parse_range_string() {
        let data = parse_range_string("(0-10)");
        assert_eq!(data.unwrap(), (0, 10));
    }

    #[test]
    fn test_parse_negative_range_string() {
        let data = parse_range_string("(-10-10)");
        assert_eq!(data.unwrap(), (-10, 10));
    }
}
