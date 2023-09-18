pub enum ArtelProgram {
    LexicalDeclaration(AlLexicalDeclaration),
}

pub enum ArtelLexicalDeclarationType {
    CONST,
    LET,
}

pub struct ArtelIdentifier(String);

pub enum ArtelType {
    PrimaryType(ArtelPrimaryType),
    Union(Vec<ArtelPrimaryType>),
}

pub enum ArtelPrimaryType {
	PredefinedType(ArtelPredefinedType),
	TypeReference(ArtelTypeReference),
	ObjectType(ArtelObjectType),
	ArrayType(Box<ArtelPrimaryType>),
	TupleType(Vec<ArtelType>),
	//TypeQuery, IDK, todo?
}

pub enum ArtelPredefinedType {
    Any,
    Number,
    Boolean,
    String,
    Void,
}

pub struct ArtelTypeReference {
    type_name: ArtelIdentifier,
    type_arguments: Vec<ArtelType>,
}

pub struct ArtelObjectType {
    // TODO
}

impl ArtelLexicalDeclarationType {
    pub fn new(s: &str) -> Self {
        match s {
            "let" => ArtelLexicalDeclarationType::LET,
            "const" => ArtelLexicalDeclarationType::CONST,
            // `var` can happen, ignore for now, TODO later
            _ => unreachable!("neither let or const found, maybe var? #TODO"),
        }
    }

    pub fn to_alstr(&self) -> &str {
        match self {
            ArtelLexicalDeclarationType::LET => "пусть",
            ArtelLexicalDeclarationType::CONST => "конст",
        }
    }
}

pub struct AlLexicalDeclaration {
    decl_type: ArtelLexicalDeclarationType,
    ident: String,
    var_type: Type, // TODO: var_type
    value: String,    // TODO expression
}

pub struct AlFunctionCall {
    callee: String,         // TODO, can also be expession
    arguments: Vec<String>, // todo: Vec of expression
}

pub enum AlExpression {
    Number(AlNumber),
}

pub struct AlNumber {
    num: String,
}
