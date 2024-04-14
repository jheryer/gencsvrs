use crate::util::fake;
use crate::util::schema::Schema;
use polars::prelude::*;
use rand::Rng;
use regex::Regex;
use std::error::Error; // rand crate is required for random number generation

type RangeParseResult = Result<(i32, i32), Box<dyn Error>>;
type DataFrameResult = Result<DataFrame, Box<dyn Error>>;
type DeleteTargetResult = Result<Vec<i32>, Box<dyn Error>>;

fn data_frame_from_file(path: &str) -> DataFrameResult {
    let mut file = std::fs::File::open(path)?;
    let df = ParquetReader::new(&mut file).finish().unwrap();
    Ok(df)
}

pub fn create_dataframe(
    schema: Vec<Schema>,
    size: usize,
    append_target: Option<String>,
    delete_target: Option<String>,
) -> DataFrameResult {
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
            "PRICE" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_price),
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
            "FIRST_NAME" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_first_name),
            ),
            "LAST_NAME" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_last_name),
            ),
            "SSN" => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::fake_ssn),
            ),
            _ => Series::new(
                element.name.as_str(),
                build_data_vector(size, fake::unknown_string),
            ),
        };

        cols.push(col);
    }

    let data_frame = match append_target {
        Some(file) => {
            let mut target_data_frame = data_frame_from_file(file.as_str())?;
            let df = DataFrame::new(cols).unwrap();

            match target_data_frame.extend(&df) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error extending DataFrame: {}", e);
                    return Err("error".into());
                }
            }
            target_data_frame
        }
        None => DataFrame::new(cols).unwrap(),
    };

    let data_frame = match delete_target {
        Some(target) => {
            let delete_target = parse_delete_target(target.as_str(), size)?;
            println!("Deleting: {:?}", delete_target);
            let mut new_df = data_frame.clone();
            for index in delete_target {
                new_df = filter_by_index(new_df, index);
            }
            new_df
        }
        None => data_frame,
    };

    Ok(data_frame)
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

fn parse_delete_target(text: &str, rows: usize) -> DeleteTargetResult {
    let mut results = Vec::new();

    if text == "random" {
        let mut rng = rand::thread_rng();
        let random_count = rng.gen_range(0..=rows);
        let random_numbers: Vec<i32> = (0..random_count)
            .map(|_| rng.gen_range(0..=rows) as i32)
            .collect();
        return Ok(random_numbers);
    }

    if let Ok(num) = text.parse::<i32>() {
        return Ok(vec![num]);
    }

    let re = Regex::new(r"^(\-?\d+)-(\-?\d+)$").unwrap();
    if let Some(caps) = re.captures(text) {
        let lower: i32 = caps.get(1).unwrap().as_str().parse()?;
        let upper: i32 = caps.get(2).unwrap().as_str().parse()?;
        return Ok((lower..=upper).collect()); // Create a range
    }

    for num_str in text.split(',') {
        let num = num_str.trim().parse::<i32>()?;
        results.push(num);
    }

    if results.len() > 0 {
        return Ok(results);
    }

    Err(format!("Error parsing delete target: {}", text).into())
}

fn filter_by_index(df: DataFrame, index: i32) -> DataFrame {
    let temp_col = build_incremental_int(df.height() as i32, 0, (df.height()) as i32);
    let col_name = fake::fake_uuid();
    let temp_series = Series::new(col_name.as_str(), temp_col);

    let new_df = df
        .lazy()
        .with_columns([temp_series.lit()])
        .filter(col(col_name.as_str()).neq(lit(index)))
        .drop([col_name.as_str()])
        .collect()
        .unwrap();

    return new_df;
}

mod test {

    #![allow(unused_imports)]
    use polars::chunked_array::iterator::par;

    use super::*;

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
            Schema {
                name: String::from("col3"),
                datatype: String::from("LOREM_WORD"),
                modifier: None,
            },
        ];

        let df = create_dataframe(schema.clone(), 10, None, Some("1,2".to_string())).unwrap();
        assert_eq!(df.shape(), (8, 3));

        let df = create_dataframe(schema.clone(), 10, None, None).unwrap();
        assert_eq!(df.shape(), (10, 3));
        // let new_df = filter_by_index(df, 2);
        // assert_eq!(new_df.shape(), (9, 3));
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

    #[test]
    fn test_delete_target() {
        let data = parse_delete_target("1,2,3", 10);
        assert_eq!(data.unwrap(), vec![1, 2, 3]);

        let data = parse_delete_target("1-3", 10);
        assert_eq!(data.unwrap(), vec![1, 2, 3]);

        let data = parse_delete_target("5", 10);
        assert_eq!(data.unwrap(), vec![5]);

        let data = parse_delete_target("random", 10);
        assert!(data.unwrap().len() > 1);

        let bad_result = parse_delete_target("xyz", 10);
        assert!(bad_result.is_err());

        let bad_result = parse_delete_target("100-2,3", 10);
        assert!(bad_result.is_err());

        let bad_result = parse_delete_target("", 10);
        assert!(bad_result.is_err());
    }
}
