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
    /// Generate relationally-consistent multi-table data from a Mermaid ER diagram
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
    /// Emit dialect-correct DDL and load-command files next to the data file
    #[arg(long, value_enum)]
    target: Option<synthtab::Dialect>,
    /// Suppress DDL file emission when --target is set
    #[arg(long)]
    no_ddl: bool,
    /// Suppress load-command file emission when --target is set
    #[arg(long)]
    no_load: bool,
}

#[derive(CLAPArgs)]
struct ErArgs {
    /// Path to a Mermaid `erDiagram` source file
    file: String,
    /// Default row count per entity
    #[arg(short, long, default_value_t = 10)]
    rows: usize,
    /// Per-entity row count override, repeatable: --rows-per ORDER=5000
    #[arg(long = "rows-per", value_parser = parse_rows_per)]
    rows_per: Vec<(String, usize)>,
    /// Output directory
    #[arg(short, long, default_value = "./out")]
    out: PathBuf,
    /// Output format
    #[arg(short = 'F', long, value_enum, default_value_t = ErFormat::Csv)]
    format: ErFormat,
    /// Emit dialect-correct DDL and load-command files next to each entity file
    #[arg(long, value_enum)]
    target: Option<synthtab::Dialect>,
    /// Suppress DDL file emission when --target is set
    #[arg(long)]
    no_ddl: bool,
    /// Suppress load-command file emission when --target is set
    #[arg(long)]
    no_load: bool,
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
        Some(Command::Er(args)) => synthtab::run_er(
            &args.file,
            args.rows,
            args.rows_per,
            args.out,
            match args.format {
                ErFormat::Csv => synthtab::ErFormat::Csv,
                ErFormat::Parquet => synthtab::ErFormat::Parquet,
            },
            args.target,
            args.no_ddl,
            args.no_load,
        ),
        None => synthtab::run(
            cli.flat.schema,
            cli.flat.rows,
            cli.flat.file_target,
            cli.flat.csv,
            cli.flat.parquet,
            cli.flat.append_target,
            cli.flat.delete_target,
            cli.flat.target,
            cli.flat.no_ddl,
            cli.flat.no_load,
        ),
    };
    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
