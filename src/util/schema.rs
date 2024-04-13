pub struct Schema {
    pub name: String,
    pub datatype: String,
    pub modifier: Option<String>,
}

impl Schema {
    pub fn from_string(input: &str) -> Option<Schema> {
        let input = input.replace(" ", "");
        let parts: Vec<&str> = input.split(':').collect();
        if !(parts.len() == 2 || parts.len() == 3) {
            println!("Bad Schema: {:?} is invalid", parts);
            None
        } else {
            Some(Schema {
                name: parts[0].trim().to_string(),
                datatype: parts[1].trim().to_string(),
                modifier: match parts.get(2) {
                    Some(xyz) => Some(xyz.trim().to_string()),
                    None => None,
                },
            })
        }
    }
}

pub fn parse_schema(input: &str) -> Vec<Schema> {
    let trimmed_input = input.trim_end_matches(',');
    let schema: Vec<Schema> = trimmed_input
        .split(',')
        .filter_map(|column_str| Schema::from_string(column_str))
        .collect();
    return schema;
}

pub fn default_schema() -> Vec<Schema> {
    vec![
        Schema {
            name: String::from("col1"),
            datatype: String::from("VALUE"),
            modifier: None,
        },
        Schema {
            name: String::from("col2"),
            datatype: String::from("VALUE"),
            modifier: None,
        },
        Schema {
            name: String::from("col3"),
            datatype: String::from("VALUE"),
            modifier: None,
        },
        Schema {
            name: String::from("col4"),
            datatype: String::from("VALUE"),
            modifier: None,
        },
    ]
}

#[cfg(test)]
mod test {

    use std::ops::Sub;

    use super::*;

    #[test]
    fn test_happy_path_schema_parser() {
        let input = "col1:STRING, col2:INT, col3:DATE, col4:INT_RNG:(1-  100) ";
        let subject = parse_schema(input);

        assert_eq!(4, subject.len());
        assert_eq!("col1", subject.get(0).unwrap().name);
        assert_eq!("STRING", subject.get(0).unwrap().datatype);
        assert_eq!("col2", subject.get(1).unwrap().name);
        assert_eq!("INT", subject.get(1).unwrap().datatype);
        assert_eq!("col3", subject.get(2).unwrap().name);
        assert_eq!("DATE", subject.get(2).unwrap().datatype);
        assert_eq!("col4", subject.get(3).unwrap().name);
        assert_eq!("INT_RNG", subject.get(3).unwrap().datatype);
        let modifier = subject.get(3).unwrap().modifier.as_ref().unwrap();
        assert_eq!("(1-100)", modifier);
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
}
