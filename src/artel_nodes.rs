#![allow(dead_code)]

pub trait ArtelStr {
    fn artel_str(&self, ident_level: usize) -> String;
}

fn ident(ident: usize) -> &'static str {
    // Yep, this is string of 100 spaces
    let much_space = "                                                                                                    "; // lol
    &much_space[0..ident]
}
//#[derive(Debug)]
//pub struct ArtelProgram(ArtelStatement);
pub type ArtelProgram = ArtelStatements;

type ArtelStatements = Vec<ArtelStatement>;

#[derive(Debug)]
pub enum ArtelStatement {
    LexicalDeclaration(ArtelLexicalDeclaration),
    FunctionDeclaration(ArtelFunctionDeclaration),
    ClassDeclaration(ArtelClassDeclaration),
    TypeAliasDeclaration(ArtelTypeAliasDeclaration),
    InterfaceDeclaration(ArtelInterfaceDeclaration),
    EnumDeclaration(EnumDeclaration),
    ExportStatement(Box<ArtelStatement>),
    Comment(String), // TODO
}

impl ArtelStr for ArtelStatement {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelStatement::FunctionDeclaration(fd) => fd.artel_str(ident_level),
            ArtelStatement::EnumDeclaration(r#enum) => r#enum.artel_str(ident_level),
            ArtelStatement::InterfaceDeclaration(interface) => interface.artel_str(ident_level),
            ArtelStatement::ClassDeclaration(class) => class.artel_str(ident_level),
            ArtelStatement::ExportStatement(exprt) => format!("{}внешнее\n{}", ident(ident_level), exprt.artel_str(ident_level + 2)),
            ArtelStatement::TypeAliasDeclaration(typealias) => typealias.artel_str(ident_level),
            ArtelStatement::Comment(comment) => format!("{}//{}", ident(ident_level), comment),
            _ => todo!("{self:?}"),
        }
    }
}

#[derive(Debug)]
pub struct EnumDeclaration {
    name: ArtelIdentifier,
    items: Vec<EnumItem>,
}

impl ArtelStr for EnumDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        str.push_str("тип ");
        str.push_str(&self.name.0);
        str.push_str(" вариант");
        str.push_str("\n");
        str.push_str(ident(ident_level));
        str.push_str("{\n");

        for enum_item in &self.items {
            str.push_str(&enum_item.artel_str(ident_level + 2));
            str.push_str("\n");
        }
        str.push_str(ident(ident_level));
        str.push_str("}");

        str
    }
}

impl EnumDeclaration {
    pub fn new(name: ArtelIdentifier, items: Vec<EnumItem>) -> Self {
        Self { name, items }
    }
}

#[derive(Debug)]
pub struct EnumItem {
    name: ArtelIdentifier,
    value: Option<String>, // TODO
}

impl ArtelStr for EnumItem {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        str.push_str(&self.name.0);
        if let Some(value) = &self.value {
            str.push_str(" = ");
            str.push_str(&value);
        }
        str
    }
}

impl EnumItem {
    pub fn new(name: ArtelIdentifier, value: Option<String>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug)]
pub enum ArtelLexicalDeclarationType {
    CONST,
    LET,
}

#[derive(Debug, Clone)]
pub struct ArtelIdentifier(String);

impl ArtelIdentifier {
    pub fn new<T: Into<String>>(name: T) -> Self {
        ArtelIdentifier(name.into())
    }
}

#[derive(Debug, Clone)]
pub struct ArtelType(pub Vec<ArtelPrimaryType>);

impl ArtelStr for ArtelType {
    fn artel_str(&self, ident_level: usize) -> String {
        assert!(!self.0.is_empty());

        let mut str = String::new();
        let mut first = true;

        fn is_questionmark(t: &ArtelPrimaryType) -> bool {
            if let ArtelPrimaryType::LiteralType(ArtelLiteralType::Null)
            | ArtelPrimaryType::LiteralType(ArtelLiteralType::Undefined) = t
            {
                true
            } else {
                false
            }
        }
        let empty_count = self
            .0
            .iter()
            .fold(0, |c, i| is_questionmark(i) as usize + c);
        let is_optional = (self.0.len() - empty_count == 1) && (empty_count > 0);

        for r#type in &self.0 {
            if is_optional && is_questionmark(&r#type) {
                continue;
            }
            if !first {
                str.push_str(" | ");
            } else {
                first = false;
            }
            str.push_str(&r#type.artel_str(0));
        }
        if is_optional {
            str.push_str("?");
        }

        str
    }
}

#[derive(Debug, Clone)]
pub enum ArtelPrimaryType {
    UnsupportedAny,
    LiteralType(ArtelLiteralType),
    PredefinedType(ArtelPredefinedType),
    TypeReference(ArtelTypeReference),
    ObjectType(ArtelObjectType),
    FunctionType(Box<ArtelFunctionDeclaration>),
    ArrayType(Box<ArtelType>),
    TupleType(Vec<ArtelType>),
    //TypeQuery, IDK, todo?
}

impl ArtelStr for ArtelPrimaryType {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelPrimaryType::UnsupportedAny => "Объект".to_owned(),
            ArtelPrimaryType::LiteralType(literal_type) => literal_type.artel_str(0),
            ArtelPrimaryType::PredefinedType(predefined_type) => predefined_type.artel_str(0),
            ArtelPrimaryType::TypeReference(type_reference) => type_reference.artel_str(0),
            ArtelPrimaryType::ObjectType(object_type) => {
                todo!()
            }
            ArtelPrimaryType::FunctionType(fun_decl) => fun_decl.artel_str_as_functype(0),
            ArtelPrimaryType::ArrayType(array_type) => {
                format!("Список<{}>", array_type.artel_str(0))
            }
            ArtelPrimaryType::TupleType(tuple_type) => {
                todo!()
            }
        }
    }
}

/// Stuff in the `<`` >` brackets, when not specified, like <T, A>
type ArtelGenericParams = Vec<ArtelTypeParameter>;

impl ArtelStr for ArtelGenericParams {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        if !self.is_empty() {
            str.push_str("<");
            let mut first = true;
            for argument in self {
                if !first {
                    str.push_str(", ");
                } else {
                    first = false;
                }
                str.push_str(&argument.artel_str(0));
            }
            str.push_str(">");
        }
        str
    }
}

#[derive(Debug, Clone)]
pub enum ArtelLiteralType {
    String(String),
    Number(String),
    Boolean(bool),
    Undefined,
    Null,
}

impl ArtelStr for ArtelLiteralType {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelLiteralType::Undefined => "пусто".to_owned(),
            ArtelLiteralType::Null => "пусто".to_owned(),
            _ => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArtelPredefinedType {
    Any,
    Number,
    Boolean,
    String,
    Void,
    Object,
}

impl ArtelStr for ArtelPredefinedType {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelPredefinedType::Any => "Объект".to_owned(),
            ArtelPredefinedType::Number => "Число".to_owned(),
            ArtelPredefinedType::Boolean => "ДаНет".to_owned(),
            ArtelPredefinedType::String => "Текст".to_owned(),
            ArtelPredefinedType::Void => "Ничего".to_owned(),
            ArtelPredefinedType::Object => "Объект".to_owned(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct ArtelTypeReference {
    type_name: ArtelIdentifier,
    type_arguments: Vec<ArtelType>,
}

impl ArtelStr for ArtelTypeReference {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(&self.type_name.0);

        if !self.type_arguments.is_empty() {
            str.push_str("<");
            let mut first = true;
            for argument in &self.type_arguments {
                if !first {
                    str.push_str(", ");
                } else {
                    first = false;
                }
                str.push_str(&argument.artel_str(0));
            }
            str.push_str(">");
        }
        str
    }
}

impl ArtelTypeReference {
    pub fn new(type_name: ArtelIdentifier, type_arguments: Vec<ArtelType>) -> Self {
        Self {
            type_name,
            type_arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtelTypeParameter {
    indentifier: ArtelIdentifier,
    constraint: Option<ArtelType>,
    default: Option<ArtelType>,
}

impl ArtelStr for ArtelTypeParameter {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(&self.indentifier.0);
        if let Some(constraint) = &self.constraint {
            str.push_str(" = ");
            str.push_str(&constraint.artel_str(0));
        }
        // TODO default
        str
    }
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

impl ArtelStr for ArtelTypeAliasDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        str.push_str("тип ");
        str.push_str(&self.alias.0);
        str.push_str(&self.generic_params.artel_str(0));
        str.push_str(" = ");
        str.push_str(&self.value.artel_str(0));

        str
    }
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
pub enum ArtelAccessModifier {
    Default,
    Public,
    Private,
    Protected,
}

impl<T> From<T> for ArtelAccessModifier
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        match value.as_ref() {
            "public" => ArtelAccessModifier::Public,
            "protected" => ArtelAccessModifier::Protected,
            "private" => ArtelAccessModifier::Private,
            _ => unimplemented!(),
        }
    }
}

impl ArtelStr for ArtelAccessModifier {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelAccessModifier::Default => String::new(),
            ArtelAccessModifier::Public => "/* public */".to_owned(),
            ArtelAccessModifier::Private => "/* private */".to_owned(),
            ArtelAccessModifier::Protected => "/* protected */".to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum ArtelModifier {
    None,
    Abstract,
    Static,
}

impl ArtelStr for ArtelModifier {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelModifier::None => String::new(),
            ArtelModifier::Abstract => "/* абстрактный */".to_owned(),
            ArtelModifier::Static => "глобальный".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ClassMemberModifiers {
    access_modifier: ArtelAccessModifier,
    modifier: ArtelModifier,
}

impl ClassMemberModifiers {
    pub fn new(access_modifier: ArtelAccessModifier, modifier: ArtelModifier) -> Self {
        Self {
            access_modifier,
            modifier,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtelObjectType {
    name: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    //body: Vec<ArtelObjectMember>,
}

#[derive(Debug)]
pub enum GetterSetter {
    None,
    Get,
    Set,
}

#[derive(Debug)]
pub enum ArtelClassMember {
    Property((ClassMemberModifiers, ArtelProperty)),
    Method((ClassMemberModifiers, GetterSetter, ArtelFunctionDeclaration)),
}

impl ArtelStr for ArtelClassMember {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelClassMember::Property((m, property)) => {
                let mut str = String::new();
                str.push_str(ident(ident_level));
                let access_modifier = m.access_modifier.artel_str(0);
                if !access_modifier.is_empty() {
                    str.push_str(&access_modifier);
                    str.push_str(" ");
                }

                let modifier = m.modifier.artel_str(0);
                if !modifier.is_empty() {
                    str.push_str(&modifier);
                    str.push_str(" ");
                }
                str.push_str(&property.artel_str(0));
                str
            }

            ArtelClassMember::Method((m, getter_setter, function_declaration)) => {
                let mut str = String::new();
                str.push_str(ident(ident_level));
                let access_modifier = m.access_modifier.artel_str(0);
                if !access_modifier.is_empty() {
                    str.push_str(&access_modifier);
                    str.push_str(" ");
                }

                let modifier = m.modifier.artel_str(0);

                assert!(if let GetterSetter::None = getter_setter {
                    true
                } else {
                    false
                });
                
                str.push_str(&function_declaration.artel_str(0));
                str
            }
        }
    }
}

#[derive(Debug)]
pub struct ArtelClassDeclaration {
    name: ArtelIdentifier,
    extends: Option<(ArtelIdentifier, ArtelGenericParams)>,
    implements: Vec<ArtelType>,
    is_abstract: bool,
    generic_params: ArtelGenericParams,
    body: Vec<ArtelClassMember>,
}

impl ArtelClassDeclaration {
    pub fn new(
        name: ArtelIdentifier,
        extends: Option<(ArtelIdentifier, ArtelGenericParams)>,
        implements: Vec<ArtelType>,
        is_abstract: bool,
        generic_params: ArtelGenericParams,
        body: Vec<ArtelClassMember>,
    ) -> Self {
        Self {
            name,
            extends,
            implements,
            is_abstract,
            generic_params,
            body,
        }
    }
}

impl ArtelStr for ArtelClassDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        str.push_str("тип ");
        str.push_str(&self.name.0);
        str.push_str(&self.generic_params.artel_str(0));

        if self.is_abstract {
            str.push_str(" = /* абстрактный */ объект");
        } else {
            str.push_str(" = объект");
        }

        if let Some((ident, params)) = &self.extends {
            str.push_str(" на основе ");
            str.push_str(&ident.0);
            str.push_str(&params.artel_str(0));
        }

        if !self.implements.is_empty() {
            // TODO
            let mut first = self.extends.is_none();
            for t in &self.implements {
                if !first {
                    str.push_str(", ");
                } else {
                    first = true
                }
                str.push_str(&t.artel_str(0));
            }
        }

        str.push_str("\n");
        str.push_str(ident(ident_level));
        str.push_str("{\n");

        let mut custom_properties = Vec::new();


        for member in &self.body {
            let is_custom_property = if let ArtelClassMember::Method((m, GetterSetter::None, function_declaration)) = member {
                false
            } else {
                true
            };

            if is_custom_property {
                custom_properties.push(member);
                continue;
            }

            str.push_str(&member.artel_str(ident_level + 2));
            str.push_str("\n");
        }

        /* processing getters and setters */
        /* another evil, 1 = getter, 2 = setter, 3 = complete */
        let mut done_props: Vec<(String, ArtelType, i32)> = Vec::new();


        for member in custom_properties {
            let ArtelClassMember::Method((m, gettersetter, function_declaration)) = member else { unreachable!() };
            let name = &function_declaration.name.0;

            let r#type;
            let num = match gettersetter {
                GetterSetter::Get => {
                    r#type = function_declaration.return_type.clone().unwrap();
                    1}
                GetterSetter::Set => {
                    r#type = function_declaration.arguments[0].r#type.clone();
                    2}
                _ => unreachable!(),
            };

            for item in &mut done_props {
                if &item.0 == name {
                    item.2 += num;
                }
            }

            done_props.push((name.clone(), r#type, num));
        }

        for custom_prop in done_props {
            let kek = ArtelProperty::new(dbg!(custom_prop.2) == 1, ArtelIdentifier(custom_prop.0), custom_prop.1);
            str.push_str(&kek.artel_str(ident_level + 2));
            str.push_str("\n");
        }

        str.push_str(ident(ident_level));
        str.push_str("}");

        str
    }
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

#[derive(Debug, Clone)]
pub struct ArtelFunctionArgument {
    name: ArtelIdentifier,
    r#type: ArtelType,
    default_value: Option<ArtelExpression>,
}

impl ArtelStr for ArtelFunctionArgument {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(&self.name.0);
        str.push_str(": ");
        str.push_str(&self.r#type.artel_str(0));
        if let Some(default_value) = &self.default_value {
            str.push_str(" = ");
            str.push_str(&default_value.artel_str(0));
        }
        str
    }
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

#[derive(Debug, Clone)]
pub struct ArtelFunctionDeclaration {
    r#async: bool,
    name: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    arguments: Vec<ArtelFunctionArgument>,
    return_type: Option<ArtelType>,
}

impl ArtelStr for ArtelFunctionDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        if self.r#async {
            str.push_str("параллельная ");
        }

        // Evil hack
        if self.name.0 == "constructor" {
            str.push_str("при создании");
        } else {
            str.push_str("операция ");
            str.push_str(&self.name.0);
            str.push_str(&self.generic_params.artel_str(0));
        }
        str.push_str(&self.arguments.artel_str(0));
        if let Some(return_type) = &self.return_type {
            str.push_str(": ");
            str.push_str(&return_type.artel_str(0));
        }
        str
    }
}

impl ArtelFunctionDeclaration {
    pub fn new(
        r#async: bool,
        name: ArtelIdentifier,
        generic_params: ArtelGenericParams,
        arguments: Vec<ArtelFunctionArgument>,
        return_type: Option<ArtelType>,
    ) -> Self {
        Self {
            r#async,
            name,
            generic_params,
            arguments,
            return_type,
        }
    }

    pub fn artel_str_as_functype(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str("операция");
        str.push_str(&self.generic_params.artel_str(0));
        str.push_str(&self.arguments.artel_str(0));
        if let Some(return_type) = &self.return_type {
            str.push_str(": ");
            str.push_str(&return_type.artel_str(0));
        }

        str
    }
}

impl ArtelStr for Vec<ArtelFunctionArgument> {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        if !self.is_empty() {
            str.push_str("(");
            let mut first = true;
            for argument in self {
                if !first {
                    str.push_str(", ");
                } else {
                    first = false;
                }
                str.push_str(&argument.artel_str(0));
            }
            str.push_str(")");
        }
        str
    }
}

#[derive(Debug)]
pub struct ArtelInterfaceDeclaration {
    name: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    body: Vec<ArtelInterfaceMember>,
}

impl ArtelStr for ArtelInterfaceDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        str.push_str("тип ");
        str.push_str(&self.name.0);
        str.push_str(&self.generic_params.artel_str(0));
        str.push_str(" = интерфейс");
        str.push_str("\n");

        str.push_str(ident(ident_level));
        str.push_str("{\n");

        for member in &self.body {
            str.push_str(&member.artel_str(ident_level + 2));
            str.push_str("\n");
        }

        str.push_str(ident(ident_level));
        str.push_str("}\n");
        str
    }
}

impl ArtelInterfaceDeclaration {
    pub fn new(
        name: ArtelIdentifier,
        generic_params: ArtelGenericParams,
        body: Vec<ArtelInterfaceMember>,
    ) -> Self {
        Self {
            name,
            generic_params,
            body,
        }
    }
}

#[derive(Debug)]
pub enum ArtelInterfaceMember {
    Property(ArtelProperty),
    Method(ArtelFunctionDeclaration),
}

impl ArtelStr for ArtelInterfaceMember {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelInterfaceMember::Property(p) => p.artel_str(ident_level),
            ArtelInterfaceMember::Method(d) => d.artel_str(ident_level),
        }
    }
}

impl Default for ArtelAccessModifier {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug)]
pub struct ArtelProperty {
    r#readonly: bool,
    name: ArtelIdentifier,
    r#type: ArtelType,
}

impl ArtelStr for ArtelProperty {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(ident(ident_level));
        if self.r#readonly {
            str.push_str("конст ");
        }
        str.push_str(&self.name.0);
        str.push_str(": ");
        str.push_str(&self.r#type.artel_str(0));

        str
    }
}

impl ArtelProperty {
    pub fn new(r#readonly: bool, name: ArtelIdentifier, r#type: ArtelType) -> Self {
        Self {
            r#readonly,
            name,
            r#type,
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

#[derive(Debug, Clone)]
pub struct ArtelExpression(pub String);

impl ArtelStr for ArtelExpression {
    fn artel_str(&self, ident_level: usize) -> String {
        self.0.clone() // TODO
    }
}

#[derive(Debug)]
pub struct AlNumber {
    num: String,
}
