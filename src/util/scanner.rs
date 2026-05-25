//! Lexical scanner for the Mermaid `erDiagram` subset documented in
//! `.claude/prds/er-diagram-generator.prd.md` §6.
//!
//! Emits a flat `Vec<(line, Token)>` so the parser (M2) can validate structure
//! without re-tracking line numbers. All errors are line-attributed per PRD §7
//! and propagated via `ScanError` — no panics on malformed input.

use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Keyword(String),
    Ident(String),
    LBrace,
    RBrace,
    Colon,
    Cardinality(String),
    StringLit(String),
    Comment(String),
    Newline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ScanError {}

pub(crate) fn scan(input: &str) -> Result<Vec<(usize, Token)>, ScanError> {
    let mut tokens = Vec::new();

    for (idx, raw_line) in input.split('\n').enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim();

        if trimmed.is_empty() {
            tokens.push((line_no, Token::Newline));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("%%") {
            tokens.push((line_no, Token::Comment(rest.trim().to_string())));
            tokens.push((line_no, Token::Newline));
            continue;
        }

        scan_line(line, line_no, &mut tokens)?;
        tokens.push((line_no, Token::Newline));
    }

    Ok(tokens)
}

fn scan_line(line: &str, line_no: usize, out: &mut Vec<(usize, Token)>) -> Result<(), ScanError> {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\r' => i += 1,
            '{' => {
                out.push((line_no, Token::LBrace));
                i += 1;
            }
            ':' => {
                out.push((line_no, Token::Colon));
                i += 1;
            }
            '"' => {
                let (lit, consumed) = scan_string_lit(&chars[i..], line_no)?;
                out.push((line_no, Token::StringLit(lit)));
                i += consumed;
            }
            '}' => {
                if matches!(chars.get(i + 1), Some('o') | Some('|')) {
                    let (card, consumed) = scan_cardinality(&chars[i..], line_no)?;
                    out.push((line_no, Token::Cardinality(card)));
                    i += consumed;
                } else {
                    out.push((line_no, Token::RBrace));
                    i += 1;
                }
            }
            '|' => {
                let (card, consumed) = scan_cardinality(&chars[i..], line_no)?;
                out.push((line_no, Token::Cardinality(card)));
                i += consumed;
            }
            '.' => {
                if matches!(chars.get(i + 1), Some('.')) {
                    return Err(ScanError {
                        line: line_no,
                        message: format!(
                            "line {line_no}: non-identifying relationships ('..') not supported; use '--'"
                        ),
                    });
                }
                return Err(ScanError {
                    line: line_no,
                    message: format!("line {line_no}: unexpected character '.'"),
                });
            }
            'o' if matches!(chars.get(i + 1), Some('{') | Some('|')) => {
                let (card, consumed) = scan_cardinality(&chars[i..], line_no)?;
                out.push((line_no, Token::Cardinality(card)));
                i += consumed;
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && is_ident_continue(chars[i]) {
                    i += 1;
                }
                let lexeme: String = chars[start..i].iter().collect();
                if lexeme == "erDiagram" {
                    out.push((line_no, Token::Keyword(lexeme)));
                } else {
                    out.push((line_no, Token::Ident(lexeme)));
                }
            }
            other => {
                return Err(ScanError {
                    line: line_no,
                    message: format!("line {line_no}: unexpected character '{other}'"),
                });
            }
        }
    }

    Ok(())
}

fn is_ident_continue(c: char) -> bool {
    // Allow `(`/`)` so parameterised types like `varchar(255)` become a single
    // identifier token; the parser (M2) strips the parens per PRD §6.3.
    c.is_alphanumeric() || c == '_' || c == '-' || c == '(' || c == ')'
}

fn scan_string_lit(chars: &[char], line: usize) -> Result<(String, usize), ScanError> {
    let mut s = String::new();
    let mut i = 1;
    while i < chars.len() && chars[i] != '"' {
        s.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() {
        return Err(ScanError {
            line,
            message: format!("line {line}: unterminated string literal"),
        });
    }
    Ok((s, i + 1))
}

/// Scan a full relationship cardinality glyph: `<left>--<right>` or rejects `<left>..<right>`.
/// `<left>` ∈ {`||`, `|o`, `o|`, `}o`, `}|`}; `<right>` ∈ {`||`, `o|`, `|o`, `o{`, `|{`}.
/// Returns the joined glyph (e.g. `||--o{`) and the number of chars consumed.
fn scan_cardinality(chars: &[char], line: usize) -> Result<(String, usize), ScanError> {
    if chars.len() < 2 {
        return Err(ScanError {
            line,
            message: format!("line {line}: incomplete cardinality glyph"),
        });
    }
    let left: String = chars[..2].iter().collect();
    match left.as_str() {
        "||" | "|o" | "o|" | "}o" | "}|" => {}
        _ => {
            return Err(ScanError {
                line,
                message: format!(
                    "line {line}: unrecognized cardinality '{left}'; supported left halves: ||, |o, o|, }}o, }}|"
                ),
            });
        }
    }

    if chars.len() < 4 {
        return Err(ScanError {
            line,
            message: format!("line {line}: cardinality '{left}' missing '--' connector"),
        });
    }
    let connector: String = chars[2..4].iter().collect();
    match connector.as_str() {
        "--" => {}
        ".." => {
            return Err(ScanError {
                line,
                message: format!(
                    "line {line}: non-identifying relationships ('..') not supported; use '--'"
                ),
            });
        }
        _ => {
            return Err(ScanError {
                line,
                message: format!(
                    "line {line}: expected '--' after cardinality '{left}', got '{connector}'"
                ),
            });
        }
    }

    if chars.len() < 6 {
        return Err(ScanError {
            line,
            message: format!("line {line}: incomplete cardinality after '--'"),
        });
    }
    let right: String = chars[4..6].iter().collect();
    match right.as_str() {
        "||" | "o|" | "|o" | "o{" | "|{" => {}
        _ => {
            return Err(ScanError {
                line,
                message: format!(
                    "line {line}: unrecognized cardinality '{right}'; supported right halves: ||, |o, o|, o{{, |{{"
                ),
            });
        }
    }

    let mut full = String::with_capacity(6);
    full.push_str(&left);
    full.push_str("--");
    full.push_str(&right);
    Ok((full, 6))
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;

    fn types_only(toks: &[(usize, Token)]) -> Vec<&Token> {
        toks.iter()
            .filter(|(_, t)| !matches!(t, Token::Newline))
            .map(|(_, t)| t)
            .collect()
    }

    #[test]
    fn empty_input_yields_single_newline() {
        let toks = scan("").unwrap();
        assert_eq!(toks.len(), 1);
        assert!(matches!(toks[0].1, Token::Newline));
    }

    #[test]
    fn blank_lines_become_newlines() {
        let toks = scan("\n\n\n").unwrap();
        for (_, t) in &toks {
            assert!(matches!(t, Token::Newline));
        }
    }

    #[test]
    fn er_diagram_keyword_is_recognized() {
        let toks = scan("erDiagram\n").unwrap();
        let non_nl = types_only(&toks);
        assert_eq!(non_nl.len(), 1);
        assert!(matches!(non_nl[0], Token::Keyword(k) if k == "erDiagram"));
    }

    #[test]
    fn comment_line_emits_comment_token() {
        let toks = scan("%% this is a comment\n").unwrap();
        let non_nl = types_only(&toks);
        assert_eq!(non_nl.len(), 1);
        match non_nl[0] {
            Token::Comment(text) => assert_eq!(text, "this is a comment"),
            _ => panic!("expected Comment"),
        }
    }

    #[test]
    fn entity_block_tokenizes_to_ident_lbrace_attrs_rbrace() {
        let src = "CUSTOMER {\n    int id PK\n    string name\n}\n";
        let toks = scan(src).unwrap();
        let non_nl = types_only(&toks);
        assert!(matches!(non_nl[0], Token::Ident(s) if s == "CUSTOMER"));
        assert!(matches!(non_nl[1], Token::LBrace));
        assert!(matches!(non_nl[2], Token::Ident(s) if s == "int"));
        assert!(matches!(non_nl[3], Token::Ident(s) if s == "id"));
        assert!(matches!(non_nl[4], Token::Ident(s) if s == "PK"));
        assert!(matches!(non_nl[5], Token::Ident(s) if s == "string"));
        assert!(matches!(non_nl[6], Token::Ident(s) if s == "name"));
        assert!(matches!(non_nl[7], Token::RBrace));
    }

    #[test]
    fn all_eight_cardinality_glyphs_from_prd_scan_cleanly() {
        let cases = [
            "||--||", "||--o{", "||--|{", "}o--o{", "}|--|{", "}o--||", "o|--||", "||--o|",
        ];
        for glyph in cases {
            let src = format!("A {glyph} B : \"label\"\n");
            let toks = scan(&src).unwrap_or_else(|e| panic!("failed to scan {glyph}: {e}"));
            let non_nl = types_only(&toks);
            assert!(matches!(non_nl[0], Token::Ident(s) if s == "A"));
            match non_nl[1] {
                Token::Cardinality(s) => assert_eq!(s, glyph),
                _ => panic!("expected Cardinality({glyph})"),
            }
            assert!(matches!(non_nl[2], Token::Ident(s) if s == "B"));
            assert!(matches!(non_nl[3], Token::Colon));
            assert!(matches!(non_nl[4], Token::StringLit(s) if s == "label"));
        }
    }

    #[test]
    fn non_identifying_connector_is_rejected_with_line_number() {
        let src = "erDiagram\nA ||..o{ B : x\n";
        let err = scan(src).unwrap_err();
        assert_eq!(err.line, 2);
        assert!(
            err.message.contains("non-identifying"),
            "got: {}",
            err.message
        );
        assert!(err.message.contains("line 2"), "got: {}", err.message);
    }

    #[test]
    fn varchar_with_parens_is_one_ident() {
        let src = "USER { varchar(255) email }\n";
        let toks = scan(src).unwrap();
        let non_nl = types_only(&toks);
        assert!(matches!(non_nl[2], Token::Ident(s) if s == "varchar(255)"));
    }

    #[test]
    fn line_numbers_track_across_lines() {
        let src = "erDiagram\n\nA {\n  int id PK\n}\n";
        let toks = scan(src).unwrap();
        let rbrace = toks
            .iter()
            .find(|(_, t)| matches!(t, Token::RBrace))
            .unwrap();
        assert_eq!(rbrace.0, 5);
    }

    #[test]
    fn hyphenated_entity_name_is_single_ident() {
        let toks = scan("NAMED-DRIVER {\n}\n").unwrap();
        let non_nl = types_only(&toks);
        assert!(matches!(non_nl[0], Token::Ident(s) if s == "NAMED-DRIVER"));
    }

    #[test]
    fn unterminated_string_literal_returns_error() {
        let src = "A ||--o{ B : \"unterminated\n";
        let err = scan(src).unwrap_err();
        assert!(err.message.contains("unterminated"), "got: {}", err.message);
    }

    #[test]
    fn unrecognized_cardinality_returns_error() {
        let src = "A ||--xx B : x\n";
        let err = scan(src).unwrap_err();
        assert!(err.message.contains("unrecognized") || err.message.contains("xx"));
    }
}
