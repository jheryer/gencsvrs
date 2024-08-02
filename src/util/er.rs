// er.rs

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::error::Error;

type RunResult<T> = Result<T, Box<dyn Error>>;

pub fn create_from_erd(filename: &str) -> RunResult<()> {
    if let Ok(lines) = read_lines(filename) {
        for line in lines 
        {
            if let Ok(line) = line {
                let tokens = tokenize(&line);
                for token in tokens {
                    println!("{}", token);
                }
            }
        }
    } else {
        return Err(format!("Unable to open file: {}", filename).into());
    }
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> 
where 
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn tokenize(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}


