//! Types in ast.
//! AstType is the main type to represen't type, which represent union of types. Union of a single
//! type represent just a type.

use super::*;
use itertools::Itertools;

/// Main sturcture for representing any type.
/// Consists of vector containing `PrimaryType`, to represent type unions.
#[derive(Debug, Clone)]
pub struct Type(pub Vec<PrimaryType>);

/// Single type declaration in a typescript.
#[derive(Debug, Clone)]
pub enum PrimaryType {
    /// Catchall type, for everything that is unsupported. Replaced with `any` during translation.
    /// `typeof Something``, ...
    UnsupportedAny(String),

    /// Literal type, such as literal number or string. `1 | 2 | 3` or `"blue" | "red" | "white"`.
    /// `undefined` and `null` are also literal types.
    ///
    /// `1`, `"hello"`, `undefined`, `null`, ...
    LiteralType(LiteralType),

    /// Predefined typescript types
    ///
    /// `number`, `string`, ...
    PredefinedType(PredefinedType),

    /// Typename with generic parameters for it
    ///
    /// `MyList<string>`, `A<B>`, ...
    TypeReference(TypeReference),

    /// Inline object type
    ///
    /// `{ name: type, readonly prop: string, method(a?: number): number}`
    ObjectType(ObjectType),

    /// `(a: string) => void`
    FunctionType(Box<FunctionDeclaration>),

    /// Wrapper type which declares array.
    ///
    /// SomeType[]
    ArrayType(Box<Type>),

    /// Wrapper type which declares readonly property.
    ///
    /// `readonly SomeType`
    ReadonlyType(Box<Type>),

    /// `(SomeType, OtherType)`
    TupleType(Vec<Type>),

    /// `SomeType is OtherType` ??? no idea lol
    PredicateType(String, Box<Type>),
}

#[derive(Debug, Clone)]
pub enum LiteralType {
    String(String),
    Number(String),
    Boolean(bool),
    Undefined,
    Null,
}

#[derive(Debug, Clone)]
pub enum PredefinedType {
    Any,
    Number,
    Boolean,
    String,
    Void,
    Object,
    Never,
    Unknown,
    Symbol,
    UniqueSymbol,
}

#[derive(Debug, Clone)]
pub struct TypeReference {
    type_name: Identifier,
    type_arguments: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct TypeParameter {
    indentifier: Identifier,
    constraint: Option<Type>,
    _default: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    body: Vec<InterfaceMember>,
}

/// Stuff in the `<` `>` brackets, when not specified, like <T, A>
pub type GenericParams = Vec<TypeParameter>;

impl Type {
    /// Check if type should be ommited if it's the only return type of the function,
    /// or should union with it be replaced with `?` instead.
    ///
    /// Like `number | undefined` -> `Число?`
    pub fn is_nothing(&self) -> bool {
        if let [PrimaryType::PredefinedType(PredefinedType::Void)
        | PrimaryType::LiteralType(LiteralType::Null)
        /*| PrimaryType::LiteralType(LiteralType::Undefined)*/] = self.0[..]
        {
            true
        } else {
            false
        }
    }

    /// Formats return type of a function, it it's not 'empty' type, returns `: TYPE`, otherwise returns empty string.
    pub fn convert_return_type(&self) -> String {
        if self.is_nothing() {
            "".to_owned()
        } else {
            [": ", &self.artel_str(0)].concat()
        }
    }
}

impl ArtelStr for Type {
    fn artel_str(&self, _ident_level: usize) -> String {
        assert!(!self.0.is_empty());
        let mut str = String::new();
        let mut first = true;

        // Code here searches for any type, that should be represented as optional in artel.
        // If we have a union with null/undefined/void, there's only single type left except null
        // types, then we append `?` to the type instead.
        fn is_questionmark(t: &PrimaryType) -> bool {
            if let PrimaryType::LiteralType(LiteralType::Null)
            | PrimaryType::LiteralType(LiteralType::Undefined)
            | PrimaryType::PredefinedType(PredefinedType::Void) = t
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

        let mut that_one_type = None;
        for r#type in &self.0 {
            if is_optional && is_questionmark(&r#type) {
                continue;
            }
            if !first {
                str.push_str(" | ");
            } else {
                first = false;
            }
            that_one_type = Some(r#type);
            str.push_str(&r#type.artel_str(0));
        }

        // can be just unwrap, idk
        if is_optional
            && !that_one_type
                .map(|t| t.is_questionmark_bydefault())
                .unwrap_or_default()
        {
            if !that_one_type
                .map(|t| t.can_be_anottated_to_left())
                .unwrap_or_default()
            {
                str.insert(0, '(');
                str.push_str(")?");
            } else {
                str.push_str("?");
            }
        }

        str
    }
}

impl PrimaryType {
    /// Formats slice of types as if they were a tuple.
    fn artel_str_tuple(tuple_type: &[Type]) -> String {
        [
            "объект { ",
            &tuple_type
                .iter()
                .enumerate()
                .map(|(i, t)| format!("_{i}: {}", &t.artel_str(0)))
                .join(", "),
            " }",
        ]
        .concat()
    }

    /// Check if type is translated to optional without any way to change it, like `?`
    pub fn is_questionmark_bydefault(&self) -> bool {
        if let PrimaryType::UnsupportedAny(_)
        | PrimaryType::PredefinedType(PredefinedType::Any)
        | PrimaryType::PredefinedType(PredefinedType::Unknown) = self
        {
            true
        } else {
            false
        }
    }

    /// Funny story, when functional type is the last type of type expression
    /// then you can't put `?` after it, cause it'll be maybe confused with
    /// `?` to the last argument, or `[]` is also usupported
    pub fn can_be_anottated_to_left(&self) -> bool {
        if let PrimaryType::FunctionType(_) = self {
            false
        } else {
            true
        }
    }
}

impl ArtelStr for PrimaryType {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            PrimaryType::UnsupportedAny(s) => format!("/*(!) {s} */ Объект?"),
            PrimaryType::LiteralType(literal_type) => literal_type.artel_str(0),
            PrimaryType::PredefinedType(predefined_type) => predefined_type.artel_str(0),
            PrimaryType::TypeReference(type_reference) => type_reference.artel_str(0),
            PrimaryType::ObjectType(object_type) => object_type.artel_str(0),
            PrimaryType::FunctionType(fun_decl) => fun_decl.artel_str_as_functype(0),
            PrimaryType::ArrayType(array_type) => {
                format!("Список<{}>", array_type.artel_str(0))
            }
            PrimaryType::TupleType(tuple_type) => Self::artel_str_tuple(tuple_type),
            PrimaryType::ReadonlyType(rtype) => {
                if rtype.0.len() > 1 {
                    format!("/*(!) защищено */ ({})", rtype.artel_str(0))
                } else {
                    format!("/*(!) защищено */ {}", rtype.artel_str(0))
                }
            }
            PrimaryType::PredicateType(predicate, r#type) => {
                format!("/*(!) {} is */ {}", predicate, r#type.artel_str(0))
            }
        }
    }
}

impl ObjectType {
    pub fn new(body: Vec<InterfaceMember>) -> Self {
        Self { body }
    }
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

impl ArtelStr for GenericParams {
    fn artel_str(&self, _ident_level: usize) -> String {
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

impl ArtelStr for LiteralType {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            LiteralType::Undefined => "пусто".to_owned(),
            LiteralType::Null => "пусто".to_owned(),
            LiteralType::String(x) => format!("\"{x}\""),
            LiteralType::Number(x) => x.clone(),
            LiteralType::Boolean(x) => if *x { "да" } else { "нет" }.to_owned(),
        }
    }
}

impl ArtelStr for PredefinedType {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            PredefinedType::Any => "/*(!) any */ Объект?".to_owned(),
            PredefinedType::Number => "Число".to_owned(),
            PredefinedType::Boolean => "ДаНет".to_owned(),
            PredefinedType::String => "Текст".to_owned(),
            PredefinedType::Void => "Ничего".to_owned(),
            PredefinedType::Object => "Объект".to_owned(),
            PredefinedType::Never => "Никогда".to_owned(),
            PredefinedType::Unknown => "/*(!) unknown */ Объект?".to_owned(),
            PredefinedType::Symbol => "Символ".to_owned(),
            PredefinedType::UniqueSymbol => "/*(!) unique symbol */ Объект".to_owned(),
        }
    }
}

impl<T> From<T> for PredefinedType
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        match value.as_ref() {
            "any" => PredefinedType::Any,
            "number" => PredefinedType::Number,
            "boolean" => PredefinedType::Boolean,
            "string" => PredefinedType::String,
            "void" => PredefinedType::Void,
            "object" => PredefinedType::Object,
            "never" => PredefinedType::Never,
            "unknown" => PredefinedType::Unknown,
            "symbol" => PredefinedType::Symbol,
            "unique symbol" => PredefinedType::UniqueSymbol,
            _ => unreachable!("{}", value.as_ref()),
        }
    }
}

impl ArtelStr for TypeReference {
    fn artel_str(&self, _ident_level: usize) -> String {
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

impl TypeReference {
    pub fn new(type_name: Identifier, type_arguments: Vec<Type>) -> Self {
        Self {
            type_name,
            type_arguments,
        }
    }
}

impl ArtelStr for TypeParameter {
    fn artel_str(&self, _ident_level: usize) -> String {
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

impl TypeParameter {
    pub fn new(
        indentifier: Identifier,
        constraint: Option<Type>,
        default: Option<Type>,
    ) -> Self {
        Self {
            indentifier,
            constraint,
            _default: default,
        }
    }
}
