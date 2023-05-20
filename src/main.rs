use clap::Parser as CLAPParser;

/*

gencsv
col0,col1,col2,col3,col4,col5,col6,col7,col8,col9
rand,rand,rand,rand,rand,rand,rand,rand,rand,rand
 */

///CLI to generate csv files
#[derive(CLAPParser)]
#[command(author,version,about,long_about=None)]
pub struct Args {
    ///Data Schema
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
