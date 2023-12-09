pub struct Schema {
    pub name: String,
    pub datatype: String,
}

impl Schema {
    pub fn from_string(input: &str) -> Option<Schema> {
        let input = input.replace(" ", "");
        let parts: Vec<&str> = input.splitn(2, ':').collect();
        if parts.len() != 2 {
            println!("Bad Schema: {:?} is invalid", parts);
            None
        } else {
            Some(Schema {
                name: parts[0].trim().to_string(),
                datatype: parts[1].trim().to_string(),
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
