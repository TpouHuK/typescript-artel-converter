use itertools::Itertools;

pub use super::*;

#[derive(Debug)]
pub struct ArtelClassDeclaration {
    name: ArtelIdentifier,
    extends: Option<(ArtelIdentifier, ArtelGenericParams)>,
    implements: Vec<Type>,
    is_abstract: bool,
    generic_params: ArtelGenericParams,
    body: Vec<ArtelClassMember>,
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

#[derive(Debug, Eq, PartialEq)]
pub enum GetterSetter {
    None,
    Get,
    Set,
}

#[derive(Debug)]
pub struct ClassMemberModifiers {
    access_modifier: ArtelAccessModifier,
    modifier: ArtelModifier,
}

#[derive(Debug)]
pub enum ArtelAccessModifier {
    Default,
    Public,
    Private,
    Protected,
}

#[derive(Debug)]
pub enum ArtelModifier {
    None,
    Abstract,
    Static,
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


impl ArtelStr for ArtelModifier {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self {
            ArtelModifier::None => String::new(),
            ArtelModifier::Abstract => "/*(!) абстрактный */".to_owned(),
            ArtelModifier::Static => "глобальный".to_owned(),
        }
    }
}
