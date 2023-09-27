use itertools::Itertools;

pub use super::*;

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
