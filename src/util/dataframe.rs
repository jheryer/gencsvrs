use crate::util::fake::build_incremental_int;
use crate::util::fake::create_column;
use crate::util::fake::fake_uuid;
use crate::util::schema::Schema;
use polars::prelude::*;
use rand::Rng;
use regex::Regex;
use std::error::Error;

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
        let col = create_column(element, size);
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
            let delete_indexes = parse_delete_target(target.as_str(), size)?;
            let mut new_df = data_frame.clone();
            new_df = filter_by_index(new_df, delete_indexes);
            new_df
        }
        None => data_frame,
    };

    Ok(data_frame)
}

fn parse_delete_target(text: &str, rows: usize) -> DeleteTargetResult {
    let mut results = Vec::new();

    if text == "random" || text == "rand" {
        let mut rng = rand::thread_rng();
        let random_count = rng.gen_range(1..=rows);
        let mut random_numbers: Vec<i32> = (0..random_count)
            .map(|_| rng.gen_range(0..=rows) as i32)
            .collect();
        random_numbers.dedup();
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

fn filter_by_index(df: DataFrame, list: Vec<i32>) -> DataFrame {
    let temp_col = build_incremental_int(df.height() as i32, 0, (df.height()) as i32);
    let col_name = fake_uuid();
    let temp_series = Series::new(col_name.as_str(), temp_col);
    let mut indexes = list.clone();
    let mut predicate = col(col_name.as_str()).neq(lit(indexes.pop().unwrap()));

    for i in list {
        predicate = predicate.clone().and(col(col_name.as_str()).neq(lit(i)));
    }

    let new_df = df
        .lazy()
        .with_columns([temp_series.lit()])
        .filter(predicate)
        .drop([col_name.as_str()])
        .collect()
        .unwrap();

    return new_df;
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_schema() -> Vec<Schema> {
        vec![
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
        ]
    }

    #[test]
    fn test_create_dataframe() {
        let schema = sample_schema();

        let df = create_dataframe(schema.clone(), 10, None, Some("1,2".to_string())).unwrap();
        assert_eq!(df.shape(), (8, 3));

        let df = create_dataframe(schema.clone(), 10, None, None).unwrap();
        assert_eq!(df.shape(), (10, 3));
    }

    #[test]
    fn test_parse_delete_target_single() {
        let r = parse_delete_target("3", 10).unwrap();
        assert_eq!(r, vec![3]);
    }

    #[test]
    fn test_parse_delete_target_comma_list() {
        let r = parse_delete_target("0,2,5", 10).unwrap();
        assert_eq!(r, vec![0, 2, 5]);
    }

    #[test]
    fn test_parse_delete_target_positive_range() {
        let r = parse_delete_target("1-4", 10).unwrap();
        assert_eq!(r, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_parse_delete_target_negative_range() {
        // -2 .. 2 inclusive should yield 5 numbers; the current ambiguous regex
        // mis-parses this and the function falls through to an error.
        let r = parse_delete_target("-2-2", 10).unwrap();
        assert_eq!(r, vec![-2, -1, 0, 1, 2]);
    }

    #[test]
    fn test_parse_delete_target_empty_returns_err() {
        assert!(parse_delete_target("", 10).is_err());
    }

    #[test]
    fn test_filter_by_index_empty_list_is_noop() {
        let df = create_dataframe(sample_schema(), 5, None, None).unwrap();
        // Must not panic on empty index list and must return all rows unchanged.
        let out = filter_by_index(df, vec![]);
        assert_eq!(out.shape(), (5, 3));
    }

    #[test]
    fn test_data_frame_from_file_bad_path_returns_err() {
        let r = data_frame_from_file("/nonexistent/path/abc.parquet");
        assert!(r.is_err());
    }

    #[test]
    fn test_data_frame_from_file_invalid_parquet_returns_err() {
        // Write garbage bytes to a temp path and try to read it as parquet.
        let path = std::env::temp_dir().join("gencsv_bad.parquet");
        std::fs::write(&path, b"not a parquet file").unwrap();
        let r = data_frame_from_file(path.to_str().unwrap());
        let _ = std::fs::remove_file(&path);
        assert!(r.is_err(), "expected Err on malformed parquet, got Ok");
    }
}
