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
    RelationshipExactlyOneLeft,
    RelationshipExactlyOneRight,
    RelationshipZoerOrMoreLeft,
    RelationshipZoerOrMoreRight,
    RelationshipOneOrMoreLeft,
    RelationshipOneOrMoreRight,
}

#[derive(Debug, PartialEq)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    line: usize,
}

impl Scanner {
    fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            current: 0,
            line: 1,
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.scan_token();
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '{' => self.add_token(TokenType::BraceOpen, "{".to_string()),
            '}' => self.add_token(TokenType::BraceClose, "}".to_string()),
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

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{:?}", token);
        }
    }
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
}
