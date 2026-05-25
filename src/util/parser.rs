//! Parser for the Mermaid `erDiagram` subset.
//!
//! Consumes the token stream from `scanner::scan` and produces an `ErdAst`
//! after enforcing every rejection rule in
//! `.claude/prds/er-diagram-generator.prd.md` §7. A successful parse
//! guarantees: header present, every entity has exactly one PK, attribute
//! types are supported, all relationship endpoints reference declared
//! entities, no cycles in the FK graph, and entity/attribute names match
//! the documented regex.

use crate::util::erd_ast::{Attribute, Cardinality, Entity, ErdAst, KeyKind, Relationship};
use crate::util::scanner::Token;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ParseError {}

/// Mermaid attribute types accepted by the parser. Maps to the gencsv generator
/// type used at row-generation time (see PRD §6.3). Mermaid `bool`/`boolean`
/// is intentionally absent: it is detected and rejected with a specific
/// "not yet supported" message per PRD §7.
pub(crate) fn mermaid_type_to_gencsv(mermaid: &str) -> Option<&'static str> {
    let base = mermaid
        .split_once('(')
        .map(|(b, _)| b)
        .unwrap_or(mermaid)
        .to_lowercase();
    match base.as_str() {
        "int" | "integer" | "bigint" => Some("INT_INC"),
        "string" | "varchar" | "text" => Some("STRING"),
        "uuid" => Some("UUID"),
        "date" => Some("DATE"),
        "datetime" | "timestamp" => Some("DATE_TIME"),
        "time" => Some("TIME"),
        "decimal" | "float" | "double" | "money" => Some("PRICE"),
        _ => None,
    }
}

/// True if `s` matches the PRD §6.2 entity-name regex `[A-Z][A-Z0-9_-]*`.
fn is_valid_entity_name(s: &str) -> bool {
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// True if `s` matches the PRD §6.2 attribute-name regex `[a-z][a-zA-Z0-9_]*`.
fn is_valid_attr_name(s: &str) -> bool {
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn parse(tokens: Vec<(usize, Token)>) -> Result<ErdAst, ParseError> {
    let mut p = Parser::new(tokens);
    p.parse_program()?;
    let ast = ErdAst {
        entities: p.entities,
        relationships: p.relationships,
    };
    validate(&ast)?;
    Ok(ast)
}

struct Parser {
    tokens: Vec<(usize, Token)>,
    pos: usize,
    entities: Vec<Entity>,
    relationships: Vec<Relationship>,
}

impl Parser {
    fn new(tokens: Vec<(usize, Token)>) -> Self {
        Self {
            tokens,
            pos: 0,
            entities: Vec::new(),
            relationships: Vec::new(),
        }
    }

    fn current(&self) -> Option<&(usize, Token)> {
        self.tokens.get(self.pos)
    }

    fn peek_after_trivia(&self) -> Option<&(usize, Token)> {
        let mut i = self.pos;
        loop {
            let t = self.tokens.get(i)?;
            if matches!(t.1, Token::Newline | Token::Comment(_)) {
                i += 1;
                continue;
            }
            return Some(t);
        }
    }

    fn skip_trivia(&mut self) {
        while matches!(
            self.tokens.get(self.pos).map(|(_, t)| t),
            Some(Token::Newline) | Some(Token::Comment(_))
        ) {
            self.pos += 1;
        }
    }

    fn bump(&mut self) -> Option<(usize, Token)> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn parse_program(&mut self) -> Result<(), ParseError> {
        self.skip_trivia();
        match self.bump() {
            Some((_, Token::Keyword(k))) if k == "erDiagram" => {}
            Some((line, t)) => {
                return Err(ParseError {
                    message: format!(
                        "line {line}: expected 'erDiagram' keyword at start of file, got {t:?}"
                    ),
                });
            }
            None => {
                return Err(ParseError {
                    message: "expected 'erDiagram' keyword at start of file".to_string(),
                });
            }
        }

        loop {
            self.skip_trivia();
            if self.at_end() {
                break;
            }
            self.parse_statement()?;
        }
        Ok(())
    }

    fn parse_statement(&mut self) -> Result<(), ParseError> {
        let (start_line, head) = match self.peek_after_trivia().cloned() {
            Some(t) => t,
            None => return Ok(()),
        };
        let name = match head {
            Token::Ident(n) => n,
            other => {
                return Err(ParseError {
                    message: format!(
                        "line {start_line}: expected entity name or relationship, got {other:?}"
                    ),
                });
            }
        };
        self.bump();

        let next = self.peek_after_trivia().cloned();
        match next {
            Some((_, Token::LBrace)) => self.parse_entity_body(name, start_line),
            Some((_, Token::Cardinality(g))) => self.parse_relationship_tail(name, g, start_line),
            Some((line, t)) => Err(ParseError {
                message: format!(
                    "line {line}: expected '{{' or relationship cardinality after entity name '{name}', got {t:?}"
                ),
            }),
            None => Err(ParseError {
                message: format!("line {start_line}: unexpected end of input after '{name}'"),
            }),
        }
    }

    fn parse_entity_body(&mut self, name: String, start_line: usize) -> Result<(), ParseError> {
        if !is_valid_entity_name(&name) {
            return Err(ParseError {
                message: format!(
                    "line {start_line}: entity name '{name}' must start with an uppercase letter and match [A-Z][A-Z0-9_-]*"
                ),
            });
        }

        match self.bump() {
            Some((_, Token::LBrace)) => {}
            other => {
                return Err(ParseError {
                    message: format!(
                        "line {start_line}: expected '{{' after entity name, got {other:?}"
                    ),
                });
            }
        }

        let mut attributes: Vec<Attribute> = Vec::new();
        loop {
            self.skip_trivia();
            match self.current().cloned() {
                Some((_, Token::RBrace)) => {
                    self.bump();
                    break;
                }
                Some((line, Token::Ident(ty))) => {
                    self.bump();
                    let (n_line, n_tok) = self.bump().ok_or_else(|| ParseError {
                        message: format!("line {line}: expected attribute name after type '{ty}'"),
                    })?;
                    let attr_name = match n_tok {
                        Token::Ident(s) => s,
                        other => {
                            return Err(ParseError {
                                message: format!(
                                    "line {n_line}: expected attribute name, got {other:?}"
                                ),
                            });
                        }
                    };

                    if !is_valid_attr_name(&attr_name) {
                        return Err(ParseError {
                            message: format!(
                                "line {line}: attribute name '{attr_name}' must start with a lowercase letter and match [a-z][a-zA-Z0-9_]*"
                            ),
                        });
                    }

                    let mut key: Option<KeyKind> = None;
                    if let Some((_, Token::Ident(k))) = self.current().cloned() {
                        match k.as_str() {
                            "PK" => {
                                key = Some(KeyKind::Pk);
                                self.bump();
                            }
                            "FK" => {
                                key = Some(KeyKind::Fk);
                                self.bump();
                            }
                            "UK" => {
                                key = Some(KeyKind::Uk);
                                self.bump();
                            }
                            _ => {}
                        }
                    }

                    if let Some((cl, Token::StringLit(_))) = self.current().cloned() {
                        return Err(ParseError {
                            message: format!(
                                "line {cl}: attribute comments are not supported in v1; remove the trailing string"
                            ),
                        });
                    }

                    let lower = ty.to_lowercase();
                    let lower_base = lower.split_once('(').map(|(b, _)| b).unwrap_or(&lower);
                    if lower_base == "bool" || lower_base == "boolean" {
                        return Err(ParseError {
                            message: format!(
                                "line {line}: type 'boolean' not yet supported (Phase 6); use 'string' as a workaround"
                            ),
                        });
                    }

                    if mermaid_type_to_gencsv(&ty).is_none() {
                        return Err(ParseError {
                            message: format!(
                                "line {line}: unknown type '{ty}'; supported types: int, string, uuid, date, datetime, decimal, time (see docs/ERD.md §Types)"
                            ),
                        });
                    }

                    attributes.push(Attribute {
                        name: attr_name,
                        data_type: ty,
                        key,
                        line,
                    });
                }
                Some((line, t)) => {
                    return Err(ParseError {
                        message: format!(
                            "line {line}: expected attribute or '}}' in entity '{name}', got {t:?}"
                        ),
                    });
                }
                None => {
                    return Err(ParseError {
                        message: format!(
                            "entity '{name}': unexpected end of input (missing closing '}}')"
                        ),
                    });
                }
            }
        }

        if self.entities.iter().any(|e| e.name == name) {
            return Err(ParseError {
                message: format!("line {start_line}: duplicate entity '{name}'"),
            });
        }

        self.entities.push(Entity {
            name,
            attributes,
            line: start_line,
        });
        Ok(())
    }

    fn parse_relationship_tail(
        &mut self,
        left: String,
        glyph: String,
        start_line: usize,
    ) -> Result<(), ParseError> {
        let (card_line, _) = self.bump().expect("peeked cardinality must exist");
        let cardinality = Cardinality::from_glyph(&glyph).ok_or_else(|| ParseError {
            message: format!(
                "line {card_line}: unrecognized cardinality '{glyph}'; supported: ||, |o, o|, }}o, o{{, }}|, |{{"
            ),
        })?;

        let (r_line, r_tok) = self.bump().ok_or_else(|| ParseError {
            message: format!("line {card_line}: relationship missing right entity"),
        })?;
        let right = match r_tok {
            Token::Ident(s) => s,
            other => {
                return Err(ParseError {
                    message: format!(
                        "line {r_line}: expected right entity name in relationship, got {other:?}"
                    ),
                });
            }
        };

        match self.bump() {
            Some((_, Token::Colon)) => {}
            Some((cl, t)) => {
                return Err(ParseError {
                    message: format!(
                        "line {cl}: expected ':' before relationship label, got {t:?}"
                    ),
                });
            }
            None => {
                return Err(ParseError {
                    message: format!("line {r_line}: relationship missing ':' and label"),
                });
            }
        }

        let label = match self.bump() {
            Some((_, Token::StringLit(s))) => Some(s),
            Some((_, Token::Ident(s))) => Some(s),
            Some((cl, t)) => {
                return Err(ParseError {
                    message: format!("line {cl}: expected relationship label, got {t:?}"),
                });
            }
            None => {
                return Err(ParseError {
                    message: format!("line {r_line}: relationship missing label after ':'"),
                });
            }
        };

        self.relationships.push(Relationship {
            left,
            right,
            cardinality,
            label,
            line: start_line,
        });
        Ok(())
    }
}

fn validate(ast: &ErdAst) -> Result<(), ParseError> {
    for e in &ast.entities {
        let pks: Vec<&Attribute> = e
            .attributes
            .iter()
            .filter(|a| a.key == Some(KeyKind::Pk))
            .collect();
        match pks.len() {
            0 => {
                return Err(ParseError {
                    message: format!(
                        "entity '{}': no attribute marked PK; every entity must declare exactly one PK",
                        e.name
                    ),
                });
            }
            1 => {}
            n => {
                let names: Vec<String> = pks.iter().map(|a| format!("'{}'", a.name)).collect();
                return Err(ParseError {
                    message: format!(
                        "entity '{}': {n} attributes marked PK ({}); composite PKs are not supported in v1",
                        e.name,
                        names.join(", ")
                    ),
                });
            }
        }
    }

    let entity_names: HashSet<&str> = ast.entities.iter().map(|e| e.name.as_str()).collect();
    for r in &ast.relationships {
        for endpoint in [&r.left, &r.right] {
            if !entity_names.contains(endpoint.as_str()) {
                return Err(ParseError {
                    message: format!(
                        "line {}: relationship references undeclared entity '{endpoint}'",
                        r.line
                    ),
                });
            }
        }
    }

    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for e in &ast.entities {
        graph.insert(e.name.as_str(), Vec::new());
    }
    for r in &ast.relationships {
        if let Some((parent, child)) = r.cardinality.parent_child(&r.left, &r.right) {
            graph.entry(parent).or_default().push(child);
        }
    }
    if let Some(cycle) = detect_cycle(&graph) {
        return Err(ParseError {
            message: format!("cycle detected in FK graph: {}; v1 requires a DAG", cycle),
        });
    }

    Ok(())
}

fn detect_cycle(graph: &HashMap<&str, Vec<&str>>) -> Option<String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: HashMap<&str, Color> = graph.keys().map(|&k| (k, Color::White)).collect();
    let mut stack: Vec<&str> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        graph: &HashMap<&'a str, Vec<&'a str>>,
        color: &mut HashMap<&'a str, Color>,
        stack: &mut Vec<&'a str>,
    ) -> Option<String> {
        color.insert(node, Color::Gray);
        stack.push(node);
        if let Some(neighbors) = graph.get(node) {
            for &next in neighbors {
                match color.get(&next).copied().unwrap_or(Color::White) {
                    Color::Gray => {
                        let start = stack.iter().position(|n| *n == next).unwrap_or(0);
                        let mut cycle: Vec<&str> = stack[start..].to_vec();
                        cycle.push(next);
                        return Some(cycle.join(" → "));
                    }
                    Color::White => {
                        if let Some(c) = dfs(next, graph, color, stack) {
                            return Some(c);
                        }
                    }
                    Color::Black => {}
                }
            }
        }
        stack.pop();
        color.insert(node, Color::Black);
        None
    }

    let keys: Vec<&str> = graph.keys().copied().collect();
    for k in keys {
        if color.get(&k).copied() == Some(Color::White) {
            if let Some(c) = dfs(k, graph, &mut color, &mut stack) {
                return Some(c);
            }
            stack.clear();
        }
    }
    None
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;
    use crate::util::scanner::scan;

    fn parse_src(src: &str) -> Result<ErdAst, String> {
        let toks = scan(src).map_err(|e| e.message)?;
        parse(toks).map_err(|e| e.message)
    }

    #[test]
    fn parses_minimal_two_entity_diagram() {
        let src = "\
erDiagram
  CAR { int id PK }
  PERSON { int id PK }
  CAR ||--o{ PERSON : \"owns\"
";
        let ast = parse_src(src).unwrap();
        assert_eq!(ast.entities.len(), 2);
        assert_eq!(ast.entities[0].name, "CAR");
        assert_eq!(ast.relationships.len(), 1);
        assert_eq!(ast.relationships[0].cardinality, Cardinality::OneToMany);
    }

    #[test]
    fn rejects_missing_header() {
        let err = parse_src("CAR { int id PK }\n").unwrap_err();
        assert!(err.contains("erDiagram"), "got: {err}");
    }

    #[test]
    fn rejects_entity_without_pk() {
        let src = "erDiagram\nUSER { string name }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("no attribute marked PK"), "got: {err}");
        assert!(err.contains("USER"), "got: {err}");
    }

    #[test]
    fn rejects_composite_pk() {
        let src = "erDiagram\nU { int a PK\n int b PK }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("composite PKs"), "got: {err}");
    }

    #[test]
    fn rejects_undeclared_entity_in_relationship() {
        let src = "erDiagram\nA { int id PK }\nA ||--o{ B : x\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("undeclared entity 'B'"), "got: {err}");
    }

    #[test]
    fn rejects_lowercase_entity_name() {
        let src = "erDiagram\ncar { int id PK }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("uppercase"), "got: {err}");
    }

    #[test]
    fn rejects_boolean_type_with_specific_message() {
        let src = "erDiagram\nU { int id PK\n boolean active }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("'boolean' not yet supported"), "got: {err}");
    }

    #[test]
    fn rejects_unknown_attribute_type() {
        let src = "erDiagram\nU { int id PK\n moneybag salary }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("unknown type 'moneybag'"), "got: {err}");
    }

    #[test]
    fn rejects_string_attribute_comment() {
        let src = "erDiagram\nU { int id PK\n string name \"the name\" }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("attribute comments"), "got: {err}");
    }

    #[test]
    fn accepts_varchar_with_size_parameter() {
        let src = "erDiagram\nU { int id PK\n varchar(255) email }\n";
        let ast = parse_src(src).unwrap();
        assert_eq!(ast.entities[0].attributes[1].data_type, "varchar(255)");
    }

    #[test]
    fn detects_cycle_in_fk_graph() {
        let src = "\
erDiagram
  A { int id PK }
  B { int id PK }
  A ||--o{ B : x
  B ||--o{ A : y
";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("cycle detected"), "got: {err}");
    }

    #[test]
    fn many_to_many_relationship_parses() {
        let src = "\
erDiagram
  STUDENT { int id PK }
  COURSE { int id PK }
  STUDENT }o--o{ COURSE : \"enrolled in\"
";
        let ast = parse_src(src).unwrap();
        assert_eq!(ast.relationships[0].cardinality, Cardinality::ManyToMany);
        assert!(ast.relationships[0].cardinality.is_many_to_many());
    }

    #[test]
    fn duplicate_entity_is_rejected() {
        let src = "erDiagram\nA { int id PK }\nA { int id PK }\n";
        let err = parse_src(src).unwrap_err();
        assert!(err.contains("duplicate entity 'A'"), "got: {err}");
    }
}
