use clap::{Args as CLAPArgs, Parser as CLAPParser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Easily generate fake data with using the following types:
/// STRING, INT, INT_INC, INT_RNG, DIGIT, DECIMAL, DATE, TIME, DATE_TIME, NAME, ZIP_CODE, COUNTRY_CODE
/// LAT, LON, PHONE, LOREM_WORD, LOREM_SENTENCE, LOREM_PARAGRAPH, UUID , PRICE
#[derive(CLAPParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    flat: FlatArgs,
}

#[derive(Subcommand)]
enum Command {
    /// Generate relationally-consistent multi-table data from a Mermaid ER diagram (M1: scanner only)
    Er(ErArgs),
}

#[derive(CLAPArgs)]
struct FlatArgs {
    /// Data Schema "col:STRING, col2:INT, col3:TIME"
    #[arg(short, long)]
    schema: Option<String>,
    /// Output file name (required for parquet file output)
    #[arg(short, long)]
    file_target: Option<String>,
    /// Generate number of rows
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    /// CSV output
    #[arg(short, long)]
    csv: bool,
    /// Parquet output
    #[arg(short, long)]
    parquet: bool,
    /// Parquet append target
    #[arg(short, long)]
    append_target: Option<String>,
    /// Delete rows by index 1 or 1,2,3 or 1-3
    #[arg(short, long)]
    delete_target: Option<String>,
    /// Emit a dialect-correct CREATE TABLE DDL file next to the data file
    #[arg(long, value_enum)]
    target: Option<gencsv::Dialect>,
    /// Suppress DDL emission when --target is set
    #[arg(long)]
    no_ddl: bool,
}

#[derive(CLAPArgs)]
struct ErArgs {
    /// Path to a Mermaid `erDiagram` source file
    file: String,
    /// Default row count per entity (parsed for M2; unused in M1)
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    /// Per-entity row count override, repeatable: --rows-per ORDER=5000 (parsed for M2; unused in M1)
    #[arg(long = "rows-per", value_parser = parse_rows_per)]
    rows_per: Vec<(String, usize)>,
    /// Output directory (parsed for M3; unused in M1)
    #[arg(short, long, default_value = "./out")]
    out: PathBuf,
    /// Output format (parsed for M3; unused in M1)
    #[arg(short = 'F', long, value_enum, default_value_t = ErFormat::Csv)]
    format: ErFormat,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ErFormat {
    Csv,
    Parquet,
}

fn parse_rows_per(s: &str) -> Result<(String, usize), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected ENTITY=COUNT, got '{s}'"))?;
    let count: usize = v
        .parse()
        .map_err(|e| format!("invalid count in '{s}': {e}"))?;
    Ok((k.to_string(), count))
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::Er(args)) => gencsv::run_er(
            &args.file,
            args.rows,
            args.rows_per,
            args.out,
            match args.format {
                ErFormat::Csv => gencsv::ErFormat::Csv,
                ErFormat::Parquet => gencsv::ErFormat::Parquet,
            },
        ),
        None => gencsv::run(
            cli.flat.schema,
            cli.flat.rows,
            cli.flat.file_target,
            cli.flat.csv,
            cli.flat.parquet,
            cli.flat.append_target,
            cli.flat.delete_target,
            cli.flat.target,
            cli.flat.no_ddl,
        ),
    };
    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
