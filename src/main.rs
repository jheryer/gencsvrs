use clap::Parser as CLAPParser;

/// Easily generate fake CSV data with using the following types:
/// STRING, INT, DIGIT, DECIMAL, DATE, TIME, DATE_TIME, NAME, ZIP_CODE, COUNTRY_CODE
/// LAT, LON, PHONE, LOREM_WORD, LOREM_SENTENCE, LOREM_PARAGRAPH, UUID
#[derive(CLAPParser)]
#[command(author,version,about,long_about=None)]
pub struct Args {
    ///Data Schema "col:STRING, col2:INT, col3:TIME"
    #[arg(short, long)]
    schema: Option<String>,
    ///Generate number of rows
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    ///csv delimiter character
    #[arg(short, long, default_value_t = ',')]
    delimiter: char,
    ///include headers
    #[arg(short, long, default_value_t = false)]
    no_header: bool,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = gencsv::run(args.schema, args.rows, args.delimiter, args.no_header) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
