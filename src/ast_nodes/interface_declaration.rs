use itertools::Itertools;

pub use super::*;

#[derive(Debug)]
pub struct ArtelInterfaceDeclaration {
    pub name: Identifier,
    pub generic_params: GenericParams,
    pub extends: Vec<TypeReference>,
    pub body: Vec<InterfaceMember>,
}

impl ArtelInterfaceDeclaration {
    pub fn new(
        name: Identifier,
        generic_params: GenericParams,
        extends: Vec<TypeReference>,
        body: Vec<InterfaceMember>,
    ) -> Self {
        Self {
            name,
            generic_params,
            extends,
            body,
        }
    }
}

impl ArtelStr for ArtelInterfaceDeclaration {
    fn artel_str(&self, ident_level: usize) -> String {
        let mut str = String::new();
        str.push_str(indent(ident_level));
        str.push_str("тип ");
        str.push_str(&self.name.artel_str(0));
        str.push_str(&self.generic_params.artel_str(0));
        str.push_str(" = интерфейс");
        if !self.extends.is_empty() {
            str.push_str(" ");
            str.push_str(&self.extends.iter().map(|t| t.artel_str(0)).join(", "));
        }
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

#[derive(Debug, Clone)]
pub enum InterfaceMember {
    Property(ArtelProperty),
    Method(FunctionDeclaration),
    Unsupported(String),
}

impl ArtelStr for InterfaceMember {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            InterfaceMember::Property(p) => p.artel_str(ident_level),
            InterfaceMember::Method(d) => d.artel_str(ident_level),
            InterfaceMember::Unsupported(d) => format!("{}/*(!) {d}*/", indent(ident_level)),
        }
    }
}
