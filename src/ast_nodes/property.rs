pub use super::*;

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
