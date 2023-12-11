use super::*;

#[derive(Debug)]
pub struct ArtelAmbientDeclaration {
    pub is_global: bool,
    pub body: Vec<ArtelStatement>,
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
            // FIXME DRY
            if let ArtelStatement::InternalModule(_) = self.body[0] {
                format!(
                    "{ident}внешнее{global_str} {body}",
                    ident = indent(ident_level),
                    body = self.body[0].artel_str(ident_level)
                )
            } else {
                format!(
                    "{ident}внешнее{global_str}\n{ident}{body}",
                    ident = indent(ident_level),
                    body = self.body[0].artel_str(ident_level)
                )
            }
        } else {
            format!(
                "{ident}внешнее{global_str}\n{ident}{{\n{body}\n{ident}}}",
                body = self.body[0].artel_str(ident_level),
                ident = indent(ident_level)
            )
        }
    }
}
