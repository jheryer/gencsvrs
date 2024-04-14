use clap::Parser as CLAPParser;
/// Easily generate fake data with using the following types:
/// STRING, INT, INT_INC, INT_RNG, DIGIT, DECIMAL, DATE, TIME, DATE_TIME, NAME, ZIP_CODE, COUNTRY_CODE
/// LAT, LON, PHONE, LOREM_WORD, LOREM_SENTENCE, LOREM_PARAGRAPH, UUID , PRICE
#[derive(CLAPParser)]
#[command(author,version,about,long_about=None)]
pub struct Args {
    ///Data Schema "col:STRING, col2:INT, col3:TIME"
    #[arg(short, long)]
    schema: Option<String>,
    /// Output file name (rquired for parquet file output)
    #[arg(short, long)]
    file_target: Option<String>,
    ///Generate number of rows
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    ///CSV output
    #[arg(short, long)]
    csv: bool,
    ///Parquet output
    #[arg(short, long)]
    parquet: bool,
    ///Parquet append target
    #[arg(short, long)]
    append_target: Option<String>,
    ///Delete rows by index 1 or 1,2,3 or 1-3
    #[arg(short, long)]
    delete_target: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = gencsv::run(
        args.schema,
        args.rows,
        args.file_target,
        args.csv,
        args.parquet,
        args.append_target,
        args.delete_target,
    ) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
