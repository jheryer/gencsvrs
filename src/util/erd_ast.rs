//! AST for the Mermaid `erDiagram` subset documented in
//! `.claude/prds/er-diagram-generator.prd.md` §6.
//!
//! Built by `parser::parse` from a token stream and consumed by
//! `generator::generate`. All structural validation (PK count, undeclared
//! references, unsupported types, cycle detection) is performed during
//! parsing — by the time you hold an `ErdAst`, it has already satisfied
//! the PRD §6/§7 invariants and is safe to generate from.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErdAst {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
}

impl ErdAst {
    pub fn entity(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub line: usize,
}

impl Entity {
    #[allow(dead_code)] // Public API surface for downstream milestones (D5 DDL emitter)
    pub fn pk(&self) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.key == Some(KeyKind::Pk))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub data_type: String,
    pub key: Option<KeyKind>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    Pk,
    Fk,
    Uk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub left: String,
    pub right: String,
    pub cardinality: Cardinality,
    pub label: Option<String>,
    pub line: usize,
}

/// Eight supported relationship glyphs from PRD §6.4. Naming reads
/// left-to-right: `OneToMany` corresponds to `||--o{` (one A, zero-or-more B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// `||--||`  exactly-one : exactly-one
    OneToOne,
    /// `||--o|`  exactly-one : zero-or-one
    OneToOptionalOne,
    /// `o|--||`  zero-or-one : exactly-one
    OptionalOneToOne,
    /// `||--o{`  exactly-one : zero-or-more
    OneToMany,
    /// `||--|{`  exactly-one : one-or-more (every left row referenced ≥1)
    OneToMandatoryMany,
    /// `}o--||`  zero-or-more : exactly-one
    ManyToOne,
    /// `}o--o{`  many-to-many (junction table)
    ManyToMany,
    /// `}|--|{`  mandatory many-to-many (every row on both sides ≥1)
    MandatoryManyToMany,
}

impl Cardinality {
    pub fn from_glyph(g: &str) -> Option<Self> {
        match g {
            "||--||" => Some(Self::OneToOne),
            "||--o|" => Some(Self::OneToOptionalOne),
            "o|--||" => Some(Self::OptionalOneToOne),
            "||--o{" => Some(Self::OneToMany),
            "||--|{" => Some(Self::OneToMandatoryMany),
            "}o--||" => Some(Self::ManyToOne),
            "}o--o{" => Some(Self::ManyToMany),
            "}|--|{" => Some(Self::MandatoryManyToMany),
            _ => None,
        }
    }

    pub fn is_many_to_many(&self) -> bool {
        matches!(self, Self::ManyToMany | Self::MandatoryManyToMany)
    }

    /// Returns true if this cardinality requires count(child) == count(parent).
    pub fn requires_equal_counts(&self) -> bool {
        matches!(
            self,
            Self::OneToOne | Self::OneToOptionalOne | Self::OptionalOneToOne
        )
    }

    /// For non-M:N cardinalities, returns (parent_entity, child_entity) where
    /// child carries the FK column. Per PRD §6.4 the "many" side or the
    /// zero-or-one side carries the FK. Returns None for M:N (junction-only).
    pub fn parent_child<'a>(&self, left: &'a str, right: &'a str) -> Option<(&'a str, &'a str)> {
        match self {
            Self::ManyToMany | Self::MandatoryManyToMany => None,
            // ||--o{, ||--|{, ||--o|, ||--||: left is the "one" side → parent
            Self::OneToMany
            | Self::OneToMandatoryMany
            | Self::OneToOptionalOne
            | Self::OneToOne => Some((left, right)),
            // }o--||, o|--||: right is the "one" side → parent
            Self::ManyToOne | Self::OptionalOneToOne => Some((right, left)),
        }
    }
}

mod test {
    #![allow(unused_imports, dead_code)]
    use super::*;

    #[test]
    fn cardinality_round_trips_all_eight_glyphs() {
        let cases = [
            ("||--||", Cardinality::OneToOne),
            ("||--o|", Cardinality::OneToOptionalOne),
            ("o|--||", Cardinality::OptionalOneToOne),
            ("||--o{", Cardinality::OneToMany),
            ("||--|{", Cardinality::OneToMandatoryMany),
            ("}o--||", Cardinality::ManyToOne),
            ("}o--o{", Cardinality::ManyToMany),
            ("}|--|{", Cardinality::MandatoryManyToMany),
        ];
        for (glyph, expected) in cases {
            assert_eq!(Cardinality::from_glyph(glyph), Some(expected), "{glyph}");
        }
    }

    #[test]
    fn unknown_glyph_returns_none() {
        assert!(Cardinality::from_glyph("xx").is_none());
        assert!(Cardinality::from_glyph("||--xx").is_none());
    }

    #[test]
    fn parent_child_assignment_matches_prd_section_6_4() {
        assert_eq!(
            Cardinality::OneToMany.parent_child("CAR", "DRIVER"),
            Some(("CAR", "DRIVER"))
        );
        assert_eq!(
            Cardinality::ManyToOne.parent_child("ORDER", "CUSTOMER"),
            Some(("CUSTOMER", "ORDER"))
        );
        assert_eq!(
            Cardinality::ManyToMany.parent_child("STUDENT", "COURSE"),
            None
        );
    }

    #[test]
    fn entity_pk_finds_the_pk_attribute() {
        let e = Entity {
            name: "USER".into(),
            line: 1,
            attributes: vec![
                Attribute {
                    name: "name".into(),
                    data_type: "string".into(),
                    key: None,
                    line: 2,
                },
                Attribute {
                    name: "id".into(),
                    data_type: "int".into(),
                    key: Some(KeyKind::Pk),
                    line: 3,
                },
            ],
        };
        assert_eq!(e.pk().map(|a| a.name.as_str()), Some("id"));
    }
}
