pub enum AlProgram {
    LexicalDeclaration(AlLexicalDeclaration),
}

pub enum AlLexicalDeclType {
    CONST,
    LET,
}

impl AlLexicalDeclType {
    pub fn new(s: &str) -> Self {
        match s {
            "let" => AlLexicalDeclType::LET,
            "const" => AlLexicalDeclType::CONST,
            // `var` can happen, ignore for now, TODO later
            _ => unreachable!("neither let or const found, maybe var? #TODO"),
        }
    }

    pub fn to_alstr(&self) -> &str {
        match self {
            AlLexicalDeclType::LET => "пусть",
            AlLexicalDeclType::CONST => "конст",
        }
    }
}

pub struct AlLexicalDeclaration {
    decl_type: AlLexicalDeclType,
    ident: String,
    var_type: String, // TODO: var_type
    value: String,    // TODO expression
}

pub struct AlFunctionCall {
    callee: String,         // TODO, can also be expession
    arguments: Vec<String>, // todo: Vec of expression
}
