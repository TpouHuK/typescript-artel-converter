#[derive(Debug)]
pub struct ArtelProgram(ArtelStatement);

type ArtelStatements = Vec<ArtelStatement>;

#[derive(Debug)]
pub enum ArtelStatement {
    LexicalDeclaration(ArtelLexicalDeclaration),
    FunctionDeclaration(ArtelFunctionDeclaration),
    ExportStatement(Box<ArtelStatement>),
}

#[derive(Debug)]
pub enum ArtelLexicalDeclarationType {
    CONST,
    LET,
}

#[derive(Debug)]
pub struct ArtelIdentifier(String);

impl ArtelIdentifier {
    pub fn new<T: Into<String>>(name: T) -> Self {
        ArtelIdentifier(name.into())
    }
}

#[derive(Debug)]
pub enum ArtelType {
    PrimaryType(ArtelPrimaryType),
    Union(Vec<ArtelPrimaryType>),
}

#[derive(Debug)]
pub enum ArtelPrimaryType {
    UnsupportedAny,
    LiteralType(ArtelLiteralType),
    PredefinedType(ArtelPredefinedType),
    TypeReference(ArtelTypeReference),
    ObjectType(ArtelObjectType),
    ArrayType(Box<ArtelPrimaryType>),
    TupleType(Vec<ArtelType>),
    //TypeQuery, IDK, todo?
}

/// Stuff in the `<`` >` brackets, when not specified, like <T, A>
type ArtelGenericParams = Vec<ArtelTypeParameter>;

#[derive(Debug)]
pub enum ArtelLiteralType {
    String(String),
    Number(String),
    Boolean(bool),
    Undefined,
    Null,
}

#[derive(Debug)]
pub enum ArtelPredefinedType {
    Any,
    Number,
    Boolean,
    String,
    Void,
    Object,
}

impl<T> From<T> for ArtelPredefinedType
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        match value.as_ref() {
            "any" => ArtelPredefinedType::Any,
            "number" => ArtelPredefinedType::Number,
            "boolean" => ArtelPredefinedType::Boolean,
            "string" => ArtelPredefinedType::String,
            "void" => ArtelPredefinedType::Void,
            "object" => ArtelPredefinedType::Object,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct ArtelTypeReference {
    type_name: ArtelIdentifier,
    type_arguments: Vec<ArtelType>,
}

impl ArtelTypeReference {
    pub fn new(type_name: ArtelIdentifier, type_arguments: Vec<ArtelType>) -> Self {
        Self {
            type_name,
            type_arguments,
        }
    }
}

#[derive(Debug)]
pub struct ArtelTypeParameter {
    indentifier: ArtelIdentifier,
    constraint: Option<ArtelType>,
    default: Option<ArtelType>,
}

impl ArtelTypeParameter {
    pub fn new(
        indentifier: ArtelIdentifier,
        constraint: Option<ArtelType>,
        default: Option<ArtelType>,
    ) -> Self {
        Self {
            indentifier,
            constraint,
            default,
        }
    }
}

#[derive(Debug)]
pub struct ArtelTypeAliasDeclaration {
    alias: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    value: ArtelType,
}

impl ArtelTypeAliasDeclaration {
    pub fn new(
        alias: ArtelIdentifier,
        generic_params: ArtelGenericParams,
        value: ArtelType,
    ) -> Self {
        Self {
            alias,
            generic_params,
            value,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ArtelFunctionArgument {
    name: ArtelIdentifier,
    r#type: ArtelType,
    default_value: Option<ArtelExpression>,
}

impl ArtelFunctionArgument {
    pub fn new(
        name: ArtelIdentifier,
        r#type: ArtelType,
        default_value: Option<ArtelExpression>,
    ) -> Self {
        Self {
            name,
            r#type,
            default_value,
        }
    }
}

#[derive(Debug)]
pub struct ArtelFunctionDeclaration {
    name: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    arguments: Vec<ArtelFunctionArgument>,
    return_type: Option<ArtelType>,
}

impl ArtelFunctionDeclaration {
    pub fn new(
        name: ArtelIdentifier,
        generic_params: ArtelGenericParams,
        arguments: Vec<ArtelFunctionArgument>,
        return_type: Option<ArtelType>,
    ) -> Self {
        Self {
            name,
            generic_params,
            arguments,
            return_type,
        }
    }
}

#[derive(Debug)]
pub struct ArtelLexicalDeclaration {
    decl_type: ArtelLexicalDeclarationType,
    ident: String,
    var_type: ArtelType, // TODO: var_type
    value: String,       // TODO expression
}

#[derive(Debug)]
pub struct AlFunctionCall {
    callee: String,         // TODO, can also be expession
    arguments: Vec<String>, // todo: Vec of expression
}

#[derive(Debug)]
pub struct ArtelExpression(pub String);

#[derive(Debug)]
pub struct AlNumber {
    num: String,
}
