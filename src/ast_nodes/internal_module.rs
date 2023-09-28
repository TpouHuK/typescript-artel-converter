use super::*;

#[derive(Debug)]
pub struct ArtelInternalModule {
    name: Identifier,
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
            &self.statements.artel_str(ident_level + 2),
            indent(ident_level),
            "}",
        ]
        .concat()
    }
}

impl ArtelInternalModule {
    pub fn new(name: Identifier, statements: Vec<ArtelStatement>) -> Self {
        Self { name, statements }
    }
}
