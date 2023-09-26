use itertools::Itertools;
use super::*;

pub trait ArtelStr {
    fn artel_str(&self, ident_level: usize) -> String;
}

//#[derive(Debug)]
//pub struct ArtelProgram(ArtelStatement);
pub type ArtelProgram = ArtelStatements;

pub type ArtelStatements = Vec<ArtelStatement>;

impl ArtelStr for ArtelStatements {
    fn artel_str(&self, ident_level: usize) -> String {
        self.iter().map(|s| s.artel_str(ident_level)).join("\n\n")
    }
}

#[derive(Debug)]
pub enum ArtelStatement {
    LexicalDeclaration(ArtelLexicalDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ArtelClassDeclaration),
    TypeAliasDeclaration(ArtelTypeAliasDeclaration),
    InterfaceDeclaration(ArtelInterfaceDeclaration),
    EnumDeclaration(EnumDeclaration),
    ExportStatement(Box<ArtelStatement>),
    AmbientDeclaration(ArtelAmbientDeclaration),
    InternalModule(ArtelInternalModule),
    StatementBlock(ArtelStatements),
    Comment(String), // TODO
}

#[derive(Debug)]
pub struct ArtelInternalModule {
    name: ArtelIdentifier,
    statements: Vec<ArtelStatement>,
}

impl ArtelStr for ArtelInternalModule {
    fn artel_str(&self, ident_level: usize) -> String {
        let header = format!("/*(!) {} */\n", self.name.0);
        [
            indent(ident_level),
            &header,
            indent(ident_level),
            "{\n",
            &self.statements.artel_str(ident_level),
            "\n",
            indent(ident_level),
            "}",
        ]
        .concat()
    }
}

impl ArtelInternalModule {
    pub fn new(name: ArtelIdentifier, statements: Vec<ArtelStatement>) -> Self {
        Self { name, statements }
    }
}

#[derive(Debug)]
pub struct ArtelAmbientDeclaration {
    is_global: bool,
    body: Vec<ArtelStatement>,
}

impl ArtelAmbientDeclaration {
    pub fn new(is_global: bool, body: Vec<ArtelStatement>) -> Self {
        Self { is_global, body }
    }
}

impl ArtelStr for ArtelAmbientDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let global_str = self.is_global.then_some(" /*(!) global */").unwrap_or("");
        if self.body.len() == 1 {
            format!(
                "{ident}внешнее{global_str}\n{ident}{body}",
                ident = indent(ident_level),
                body = self.body[0].artel_str(ident_level)
            )
        } else {
            format!(
                "{ident}внешнее{global_str}\n{ident}{{\n{body}\n{ident}}}",
                body = self.body[0].artel_str(ident_level),
                ident = indent(ident_level)
            )
        }
    }
}

impl ArtelStr for ArtelStatement {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelStatement::FunctionDeclaration(fd) => fd.artel_str(ident_level),
            ArtelStatement::EnumDeclaration(r#enum) => r#enum.artel_str(ident_level),
            ArtelStatement::InterfaceDeclaration(interface) => interface.artel_str(ident_level),
            ArtelStatement::ClassDeclaration(class) => class.artel_str(ident_level),
            ArtelStatement::ExportStatement(exprt) => format!(
                "{}внешнее\n{}",
                indent(ident_level),
                exprt.artel_str(ident_level + 2)
            ),
            ArtelStatement::AmbientDeclaration(decl) => decl.artel_str(0),
            ArtelStatement::TypeAliasDeclaration(typealias) => typealias.artel_str(ident_level),
            ArtelStatement::Comment(comment) => set_indent(comment, ident_level),
            ArtelStatement::LexicalDeclaration(decl) => decl.artel_str(ident_level),
            ArtelStatement::InternalModule(modul) => modul.artel_str(ident_level),
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
        str.push_str(indent(ident_level));
        str.push_str("тип ");
        str.push_str(&self.name.0);
        str.push_str(" = вариант");
        str.push_str("\n");
        str.push_str(indent(ident_level));
        str.push_str("{\n");

        for enum_item in &self.items {
            str.push_str(&enum_item.artel_str(ident_level + 2));
            str.push_str("\n");
        }
        str.push_str(indent(ident_level));
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
        str.push_str(indent(ident_level));
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
    VAR,
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



#[derive(Debug)]
pub struct ArtelTypeAliasDeclaration {
    alias: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    value: Type,
}

impl ArtelStr for ArtelTypeAliasDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        [
            indent(ident_level),
            "тип ",
            &self.alias.0,
            &self.generic_params.artel_str(0),
            " = ",
            &self.value.artel_str(0),
        ]
        .concat()
    }
}

impl ArtelTypeAliasDeclaration {
    pub fn new(
        alias: ArtelIdentifier,
        generic_params: ArtelGenericParams,
        value: Type,
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
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            ArtelAccessModifier::Default => String::new(),
            ArtelAccessModifier::Public => "/*(!) public */".to_owned(),
            ArtelAccessModifier::Private => "/*(!) private */".to_owned(),
            ArtelAccessModifier::Protected => "скрыто типом".to_owned(),
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
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            ArtelModifier::None => String::new(),
            ArtelModifier::Abstract => "/*(!) абстрактный */".to_owned(),
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

impl ArtelStr for ClassMemberModifiers {
    fn artel_str(&self, _ident_level: usize) -> String {
        let mut str = String::new();

        let access_modifier = self.access_modifier.artel_str(0);
        if !access_modifier.is_empty() {
            str.push_str(&access_modifier);
            str.push_str(" ");
        }

        let modifier = self.modifier.artel_str(0);
        if !modifier.is_empty() {
            str.push_str(&modifier);
            str.push_str(" ");
        }

        str
    }
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    body: Vec<ArtelInterfaceMember>,
}

impl ArtelStr for ObjectType {
    fn artel_str(&self, _ident_level: usize) -> String {
        [
            "объект { ",
            &self
                .body
                .iter()
                .map(|p| p.artel_str(0).replace("\n", " "))
                .join("; "),
            " }",
        ]
        .concat()
    }
}

impl ObjectType {
    pub fn new(body: Vec<ArtelInterfaceMember>) -> Self {
        Self { body }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum GetterSetter {
    None,
    Get,
    Set,
}

#[derive(Debug)]
pub enum ArtelClassMember {
    Property((ClassMemberModifiers, ArtelProperty)),
    Method((ClassMemberModifiers, GetterSetter, FunctionDeclaration)),
}

struct PropertyAccessExpression {
    name: ArtelIdentifier,
    r#type: Type,
    get: bool,
    set: bool,
}

impl ArtelStr for PropertyAccessExpression {
    fn artel_str(&self, ident_level: usize) -> String {
        [
            indent(ident_level),
            &self.name.0,
            ": ",
            &self.r#type.artel_str(0),
            &if self.get {
                ["\n", indent(ident_level + 2), "при чтении { }"].concat()
            } else {
                "".to_owned()
            },
            &if self.set {
                ["\n", indent(ident_level + 2), "при записи { }"].concat()
            } else {
                "".to_owned()
            },
        ]
        .concat()
    }
}

impl ArtelClassMember {
    fn is_getter_setter(&self) -> bool {
        match self {
            ArtelClassMember::Method((
                _m,
                GetterSetter::Set | GetterSetter::Get,
                _function_declaration,
            )) => true,
            _ => false,
        }
    }

    /// Returns `PropertyAccessExpression` if the ClassMember is a Method with `get` or `set`
    /// annotation.
    fn get_property_func(&self) -> Option<PropertyAccessExpression> {
        match self {
            ArtelClassMember::Method((
                _m, // TODO, modifiers are ignored
                prop @ GetterSetter::Set | prop @ GetterSetter::Get,
                function_declaration,
            )) => {
                let r#type = match prop {
                    GetterSetter::Set => function_declaration.arguments.get(0)
                        .expect("Function annotated with `set` should have atleast single argument, which is the type of the property it access.")
                        .r#type.clone(),
                    GetterSetter::Get => function_declaration.return_type.clone()
                        .expect("Function annotated with `get` should have a return type"),
                    GetterSetter::None => unreachable!(),
                };

                Some(PropertyAccessExpression {
                    name: function_declaration.name.clone(),
                    r#type,
                    get: *prop == GetterSetter::Get,
                    set: *prop == GetterSetter::Set,
                })
            }
            _ => None,
        }
    }

    fn artel_str_property(
        modifiers: &ClassMemberModifiers,
        property: &ArtelProperty,
        ident_level: usize,
    ) -> String {
        property.artel_str_with_modifier(modifiers.artel_str(0), ident_level)
    }

    fn artel_str_method(
        modifiers: &ClassMemberModifiers,
        function_declaration: &FunctionDeclaration,
        ident_level: usize,
    ) -> String {
        function_declaration.artel_str_with_modifier(modifiers.artel_str(0), ident_level)
    }
}

impl ArtelStr for ArtelClassMember {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelClassMember::Property((modifiers, property)) => {
                Self::artel_str_property(modifiers, property, ident_level)
            }

            ArtelClassMember::Method((modifiers, getter_setter, function_declaration)) => {
                assert!(
                    *getter_setter == GetterSetter::None,
                    "Methods cannot be getters or setters"
                );
                Self::artel_str_method(modifiers, function_declaration, ident_level)
            }
        }
    }
}

#[derive(Debug)]
pub struct ArtelClassDeclaration {
    name: ArtelIdentifier,
    extends: Option<(ArtelIdentifier, ArtelGenericParams)>,
    implements: Vec<Type>,
    is_abstract: bool,
    generic_params: ArtelGenericParams,
    body: Vec<ArtelClassMember>,
}

impl ArtelClassDeclaration {
    pub fn new(
        name: ArtelIdentifier,
        extends: Option<(ArtelIdentifier, ArtelGenericParams)>,
        implements: Vec<Type>,
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

    fn artel_str_heritage(&self) -> String {
        let mut vec = Vec::new();
        if let Some((ident, params)) = &self.extends {
            vec.push([&ident.0, &params.artel_str(0), ""].concat());
        }
        vec.extend(self.implements.iter().map(|t| t.artel_str(0)));

        if !vec.is_empty() {
            " на основе ".to_owned() + &vec.join(", ")
        } else {
            "".to_owned()
        }
    }

    fn artel_str_declaration_header(&self, ident_level: usize) -> String {
        [
            indent(ident_level),
            "тип ",
            &self.name.0,
            &self.generic_params.artel_str(0),
            if self.is_abstract {
                " = /*(!) абстрактный */ объект"
            } else {
                " = объект"
            },
            &self.artel_str_heritage(),
            "\n",
            indent(ident_level),
            "{\n",
        ]
        .concat()
    }

    fn create_property_access_expressions(
        members: Vec<&ArtelClassMember>,
    ) -> Vec<PropertyAccessExpression> {
        let mut property_access_expressions: Vec<PropertyAccessExpression> = Vec::new();
        'outer: for member in members {
            let property_access_expression = member
                .get_property_func()
                .expect("Everything else is filtered before");

            for item in &mut property_access_expressions {
                // If we processed getter or setter of this property before
                if item.name == property_access_expression.name {
                    assert!(
                        item.get != property_access_expression.get
                            && item.set != property_access_expression.set,
                        "There should be only one setter and getter for the property"
                    );
                    item.get |= property_access_expression.get;
                    item.set |= property_access_expression.set;
                    continue 'outer;
                }
            }
            property_access_expressions.push(property_access_expression);
        }

        property_access_expressions
    }
}

impl ArtelStr for ArtelClassDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut getters_setters = Vec::new();
        let mut members = Vec::new();

        for member in &self.body {
            if member.is_getter_setter() {
                getters_setters.push(member);
            } else {
                members.push(member);
            }
        }
        let getters_setters = Self::create_property_access_expressions(getters_setters);

        let member_body = members.iter().map(|m| m.artel_str(ident_level + 2));
        let getter_setter_body = getters_setters.iter().map(|t| t.artel_str(ident_level + 2));
        let body_str = member_body.chain(getter_setter_body).join("\n\n");

        [
            &self.artel_str_declaration_header(ident_level),
            &body_str,
            "\n",
            indent(ident_level),
            "}",
        ]
        .concat()
    }
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

#[derive(Debug, Clone)]
pub struct ArtelFunctionArgument {
    name: ArtelIdentifier,
    r#type: Type,
    default_value: Option<ArtelExpression>,
}

impl ArtelStr for ArtelFunctionArgument {
    fn artel_str(&self, _ident_level: usize) -> String {
        let is_array_param = self.name.0.starts_with("...");
        let name = if is_array_param {
            self.name.0.strip_prefix("...").unwrap().to_owned()
        } else {
            self.name.0.clone()
        };
        [
            &name,
            ": ",
            &self.r#type.artel_str(0),
            &if let Some(default_value) = &self.default_value {
                [" = ", &default_value.artel_str(0)].concat()
            } else {
                "".to_owned()
            },
        ]
        .concat()
    }
}

impl ArtelFunctionArgument {
    pub fn new(
        name: ArtelIdentifier,
        r#type: Type,
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
pub struct FunctionDeclaration {
    r#async: bool,
    name: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    arguments: Vec<ArtelFunctionArgument>,
    return_type: Option<Type>,
}

impl FunctionDeclaration {
    fn annotation_array_param(&self, ident_level: usize) -> String {
        let any_param_array = self
            .arguments
            .iter()
            .any(|arg| arg.name.0.starts_with("..."));
        if any_param_array {
            [indent(ident_level), "#js.МассивПараметров\n"].concat()
        } else {
            String::new()
        }
    }

    fn artel_str_return_type(&self, _ident_level: usize) -> String {
        if let Some(return_type) = &self.return_type {
            Type::convert_return_type(return_type)
        } else {
            Type(vec![PrimaryType::UnsupportedAny("no_type".into())]).artel_str(0)
        }
    }
}

impl ArtelStr for FunctionDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        self.artel_str_with_modifier("".to_owned(), ident_level)
    }
}

impl FunctionDeclaration {
    pub fn new(
        r#async: bool,
        name: ArtelIdentifier,
        generic_params: ArtelGenericParams,
        arguments: Vec<ArtelFunctionArgument>,
        return_type: Option<Type>,
    ) -> Self {
        Self {
            r#async,
            name,
            generic_params,
            arguments,
            return_type,
        }
    }

    pub fn artel_str_as_functype(&self, _ident_level: usize) -> String {
        [
            "операция",
            &self.generic_params.artel_str(0),
            &self.arguments.artel_str(0),
            &self.artel_str_return_type(0),
        ]
        .concat()
    }

    fn artel_str_with_modifier(&self, modifier: String, ident_level: usize) -> String {
        let modifier = [
            &modifier,
            self.r#async.then_some("параллельная").unwrap_or(""),
        ]
        .concat();
        let modifier = (!modifier.is_empty())
            .then(|| format!("{modifier}\n{}", indent(ident_level)))
            .unwrap_or_default();

        [
            &self.annotation_array_param(ident_level),
            indent(ident_level),
            &modifier,
            // Evil hack
            &if self.name.0 == "constructor" {
                "при создании".to_owned()
            } else {
                ["операция ", &self.name.0, &self.generic_params.artel_str(0)].concat()
            },
            &self.arguments.artel_str(0),
            &self.artel_str_return_type(0),
        ]
        .concat()
    }
}

impl ArtelStr for Vec<ArtelFunctionArgument> {
    fn artel_str(&self, _ident_level: usize) -> String {
        ["(", &self.iter().map(|a| a.artel_str(0)).join(", "), ")"].concat()
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
        str.push_str(indent(ident_level));
        str.push_str("тип ");
        str.push_str(&self.name.0);
        str.push_str(&self.generic_params.artel_str(0));
        str.push_str(" = интерфейс");
        str.push_str("\n");

        str.push_str(indent(ident_level));
        str.push_str("{\n");
        str.push_str(
            &self
                .body
                .iter()
                .map(|m| m.artel_str(ident_level + 2))
                .join("\n\n"),
        );
        str.push_str("\n");
        str.push_str(indent(ident_level));
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

#[derive(Debug, Clone)]
pub enum ArtelInterfaceMember {
    Property(ArtelProperty),
    Method(FunctionDeclaration),
    Unsupported(String),
}

impl ArtelStr for ArtelInterfaceMember {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelInterfaceMember::Property(p) => p.artel_str(ident_level),
            ArtelInterfaceMember::Method(d) => d.artel_str(ident_level),
            ArtelInterfaceMember::Unsupported(d) => format!("{}/*(!) {d}*/", indent(ident_level)),
        }
    }
}

impl Default for ArtelAccessModifier {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone)]
pub struct ArtelProperty {
    r#readonly: bool,
    name: ArtelIdentifier,
    r#type: Type,
}

impl ArtelStr for ArtelProperty {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(indent(ident_level));
        let modifier = self.r#readonly.then(|| "защищено ").unwrap_or_default();
        if !modifier.is_empty() {
            str.push_str(&modifier);
            str.push_str("\n");
            str.push_str(indent(ident_level));
        }
        str.push_str(&self.name.0);
        str.push_str(": ");
        str.push_str(&self.r#type.artel_str(0));
        str
    }
}

impl ArtelProperty {
    pub fn new(r#readonly: bool, name: ArtelIdentifier, r#type: Type) -> Self {
        Self {
            r#readonly,
            name,
            r#type,
        }
    }

    pub fn artel_str_with_modifier(&self, modifier: String, ident_level: usize) -> String {
        let mut str = String::new();
        let modifier = [
            &modifier,
            self.r#readonly.then(|| "защищено").unwrap_or_default(),
        ]
        .concat();
        str.push_str(indent(ident_level));

        if !modifier.is_empty() {
            str.push_str(&modifier);
            str.push_str("\n");
            str.push_str(indent(ident_level));
        }
        str.push_str(&self.name.0);
        str.push_str(": ");
        str.push_str(&self.r#type.artel_str(0));

        str
    }
}

#[derive(Debug)]
pub struct ArtelLexicalDeclarationMember {
    ident: ArtelIdentifier,
    var_type: Type,
    value: Option<String>,
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

#[derive(Debug)]
pub struct ArtelLexicalDeclaration {
    decl_type: ArtelLexicalDeclarationType,
    body: Vec<ArtelLexicalDeclarationMember>,
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

#[derive(Debug, Clone)]
pub struct ArtelExpression(pub String);

impl ArtelStr for ArtelExpression {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self.0.as_str() {
            "false" => "нет".to_owned(),
            "true" => "да".to_owned(),
            _ => self.0.clone(),
        }
    }
}
