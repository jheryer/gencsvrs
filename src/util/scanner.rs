use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

type ScannerResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum TokenType {
    Identifier,
    DiagramStart,
    //characters
    BraceOpen,
    BraceClose,
    Space,
    //containers
    Entity,
    Label,
    //relationships
    RelationshipZeroOneLeft,
    RelationshipZeroOneRight,
    RelationshipExactlyOne,
    RelationshipZeroOrMoreLeft,
    RelationshipZeroOrMoreRight,
    RelationshipOneOrMoreLeft,
    RelationshipOneOrMoreRight,
}

#[derive(Debug, PartialEq)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.scan_token();
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '|' => {
                if self.match_next('|') {
                    self.add_token(TokenType::RelationshipExactlyOne, String::from("||"))
                } else if self.match_next('o') {
                    self.add_token(TokenType::RelationshipZeroOneLeft, String::from("|o"))
                } else if self.match_next('{') {
                    self.add_token(TokenType::RelationshipOneOrMoreRight, String::from("|{"))
                }
            }
            'o' => {
                if self.match_next('{') {
                    self.add_token(TokenType::RelationshipZeroOrMoreRight, String::from("o{"));
                } else if self.match_next('|') {
                    self.add_token(TokenType::RelationshipZeroOneRight, String::from("o|"))
                }
            }
            '{' => self.add_token(TokenType::BraceOpen, String::from("{")),
            '}' => {
                if self.match_next('o') {
                    self.add_token(TokenType::RelationshipZeroOrMoreLeft, String::from("}o"));
                } else if self.match_next('|') {
                    self.add_token(TokenType::RelationshipOneOrMoreLeft, String::from("}|"))
                } else {
                    self.add_token(TokenType::BraceClose, String::from("}"));
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            _ => {
                if c.is_alphabetic() {
                    self.scan_identifier(c)
                } else {
                    println!("how did I get here? {}", c)
                }
            }
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn scan_identifier(&mut self, start: char) {
        let mut lexeme = start.to_string();
        while !self.is_at_end() && self.peek().is_alphanumeric() {
            lexeme.push(self.advance());
        }
        self.add_token(TokenType::Identifier, lexeme);
    }

    fn add_token(&mut self, token_type: TokenType, lexeme: String) {
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
        })
    }

    fn advance(&mut self) -> char {
        let ac = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        ac
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{:?}", token);
        }
    }
}

pub fn read_file<P>(filename: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let mut content = String::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        content.push_str(&line?);
        content.push('\n');
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_braces() {
        let source = "{ }".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![
            Token {
                token_type: TokenType::BraceOpen,
                lexeme: "{".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::BraceClose,
                lexeme: "}".to_string(),
                line: 1,
            },
        ];
        scanner.print_tokens();
        assert_eq!(scanner.tokens, expected_tokens);
    }
    #[test]
    fn test_relationship_zero_or_one_left() {
        let source = "  |o  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipZeroOneLeft,
            lexeme: "|o".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }
    #[test]
    fn test_relationship_zero_or_one_right() {
        let source = "  o|  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipZeroOneRight,
            lexeme: "o|".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }

    #[test]
    fn test_relationship_exactly_one() {
        let source = "  ||  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipExactlyOne,
            lexeme: "||".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }

    #[test]
    fn test_relationship_zero_or_more_left() {
        let source = "  }o  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipZeroOrMoreLeft,
            lexeme: "}o".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }

    #[test]
    fn test_relationship_zero_or_more_right() {
        let source = "  o{  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipZeroOrMoreRight,
            lexeme: "o{".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }

    #[test]
    fn test_relationship_one_or_more_right() {
        let source = "  |{  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipOneOrMoreRight,
            lexeme: "|{".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }

    #[test]
    fn test_relationship_one_or_more_left() {
        let source = "  }|  ".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        let expected_tokens = vec![Token {
            token_type: TokenType::RelationshipOneOrMoreLeft,
            lexeme: "}|".to_string(),
            line: 1,
        }];

        assert_eq!(scanner.tokens, expected_tokens);
    }
}
