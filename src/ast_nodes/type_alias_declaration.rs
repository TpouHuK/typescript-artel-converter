use super::*;

#[derive(Debug)]
pub struct ArtelTypeAliasDeclaration {
    alias: Identifier,
    generic_params: GenericParams,
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
    pub fn new(alias: Identifier, generic_params: GenericParams, value: Type) -> Self {
        Self {
            alias,
            generic_params,
            value,
        }
    }
}
