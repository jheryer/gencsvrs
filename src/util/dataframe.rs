use crate::util::fake;
use crate::util::schema::Schema;
use polars::chunked_array::iterator::par;
use polars::prelude::*;
use regex::Regex;
use std::error::Error;

type RangeParseResult = Result<(i32, i32), Box<dyn Error>>;

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

fn build_incremental_int(size: i32, start: i32, end: i32) -> Vec<i32> {
    let end = if start - end < 0 { start + size } else { end };
    println!("start: {}, end: {}", start, end);
    (start..end as i32).collect::<Vec<i32>>()
}

fn parse_range_string(range_str: &str) -> RangeParseResult {
    // let re = Regex::new(r"\((\d+)-(\d+)\)").unwrap();
    let re = Regex::new(r"\((-?\d+)\s*-\s*(-?\d+)\)").unwrap();
    if let Some(caps) = re.captures(range_str) {
        let lower: i32 = caps.get(1).unwrap().as_str().parse()?;
        let upper: i32 = caps.get(2).unwrap().as_str().parse()?;
        Ok((lower, upper))
    } else {
        Err(format!("Error parsing {}", range_str).into())
    }
}

mod test {
    use super::*;
    use crate::util::schema::Schema;

    #[test]
    fn test_create_dataframe() {
        let schema = vec![
            Schema {
                name: String::from("col1"),
                datatype: String::from("INT"),
                modifier: None,
            },
            Schema {
                name: String::from("col2"),
                datatype: String::from("STRING"),
                modifier: None,
            },
        ];

        let df = create_dataframe(schema, 10);
        assert_eq!(df.shape(), (10, 2));
    }

    #[test]
    fn test_build_data_vector() {
        let data = build_data_vector(10, || 1);
        assert_eq!(data, vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }

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
