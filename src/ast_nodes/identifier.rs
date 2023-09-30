use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(String);

const KEYWORDS: &[&str] = &[
    "aliases",
    "and",
    "as",
    "aspect",
    "await",
    "case",
    "const",
    "dispose",
    "do",
    "else",
    "empty",
    "error",
    "external",
    "finally",
    "for",
    "from",
    "global",
    "hidden",
    "if",
    "import",
    "interface",
    "is",
    "let",
    "no",
    "nonreactive",
    "not",
    "nzon",
    "object",
    "observable",
    "operation",
    "or",
    "parallel",
    "protected",
    "reactive",
    "redefinable",
    "redefined",
    "ref",
    "return",
    "switch",
    "then",
    "transactional",
    "type",
    "variant",
    "while",
    "xor",
    "yes",
    "yield",
];

/// Artel identifier, NewType of string for no apparent reason.
impl Identifier {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Identifier(name.into())
    }

    pub fn raw(&self) -> &String {
        &self.0
    }
}

impl ArtelStr for Identifier {
    fn artel_str(&self, _ident_level: usize) -> String {
        if KEYWORDS.binary_search(&self.0.as_str()).is_ok() {
            format!("{}`", self.0)
        } else {
            self.0.clone()
        }
    }
}
