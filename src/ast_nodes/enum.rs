use super::*;

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
