pub use super::*;

#[derive(Debug)]
pub struct ArtelLexicalDeclaration {
    decl_type: ArtelLexicalDeclarationType,
    body: Vec<ArtelLexicalDeclarationMember>,
}

#[derive(Debug)]
pub enum ArtelLexicalDeclarationType {
    CONST,
    LET,
    VAR,
}

#[derive(Debug)]
pub struct ArtelLexicalDeclarationMember {
    ident: ArtelIdentifier,
    var_type: Type,
    value: Option<String>,
}

impl ArtelLexicalDeclarationType {
    pub fn new(s: &str) -> Self {
        match s {
            "let" => ArtelLexicalDeclarationType::LET,
            "const" => ArtelLexicalDeclarationType::CONST,
            "var" => ArtelLexicalDeclarationType::VAR,
            _ => unreachable!("how did we get there?"),
        }
    }
}


impl Default for ArtelAccessModifier {
    fn default() -> Self {
        Self::Default
    }
}


impl ArtelStr for ArtelLexicalDeclarationMember {
    fn artel_str(&self, _ident_level: usize) -> String {
        [
            &self.ident.0,
            ": ",
            self.var_type.artel_str(0).as_str(),
            &if let Some(value) = &self.value {
                format!(" /*(!) = {value} */")
            } else {
                "".to_owned()
            },
        ]
        .concat()
    }
}

impl ArtelLexicalDeclarationMember {
    pub fn new(ident: ArtelIdentifier, var_type: Type, value: Option<String>) -> Self {
        Self {
            ident,
            var_type,
            value,
        }
    }
}


impl ArtelLexicalDeclaration {
    pub fn new(
        decl_type: ArtelLexicalDeclarationType,
        body: Vec<ArtelLexicalDeclarationMember>,
    ) -> Self {
        Self { decl_type, body }
    }
}
impl ArtelStr for ArtelLexicalDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        self.body
            .iter()
            .map(|decl| {
                format!(
                    "{}{} {}\n",
                    indent(ident_level),
                    self.decl_type.artel_str(0),
                    decl.artel_str(0)
                )
            })
            .collect()
    }
}

impl ArtelStr for ArtelLexicalDeclarationType {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            Self::CONST => "конст".to_owned(),
            Self::LET => "пусть".to_owned(),
            Self::VAR => "/*(!) var */".to_owned(),
        }
    }
}
