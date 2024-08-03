// er.rs

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

type RunResult<T> = Result<T, Box<dyn Error>>;

enum TokenType {
    DiagramStart,
    //characters
    LeftBrace,
    RightBrace,
    Space,
    //containers
    Entity,
    Label,
    //relationships
    RelationshipZeroOneLeft,
    RelationshipZeroOneRight,
    RelationshipExactlyOneLeft,
    RelationshipExactlyOneRight,
    RelationshipZoerOrMoreLeft,
    RelationshipZoerOrMoreRight,
    RelationshipOneOrMoreLeft,
    RelationshipOneOrMoreRight,
}

struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: String,
    line: usize,
}

impl Token {
    fn new(&self, token_type: TokenType, lexeme: String, literal: String, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

pub fn create_from_erd(filename: &str) -> RunResult<()> {
    if let Ok(lines) = read_lines(filename) {
        for line in lines {
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
