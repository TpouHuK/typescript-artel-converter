use itertools::Itertools;

pub use super::*;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    r#async: bool,
    pub name: ArtelIdentifier,
    generic_params: ArtelGenericParams,
    pub arguments: Vec<ArtelFunctionArgument>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ArtelFunctionArgument {
    name: ArtelIdentifier,
    pub r#type: Type,
    default_value: Option<ArtelExpression>,
}

impl FunctionDeclaration {
    pub fn annotation_array_param(&self, ident_level: usize) -> String {
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

    pub fn artel_str_return_type(&self, _ident_level: usize) -> String {
        if let Some(return_type) = &self.return_type {
            return_type.convert_return_type()
        } else {
            Type(vec![PrimaryType::UnsupportedAny("no_type".into())]).convert_return_type()
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

    pub fn artel_str_with_modifier(&self, modifier: String, ident_level: usize) -> String {
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
