pub use super::*;
use itertools::Itertools;

pub type ArtelProgram = ArtelStatements;
pub type ArtelStatements = Vec<ArtelStatement>;

impl ArtelStr for ArtelStatements {
    fn artel_str(&self, ident_level: usize) -> String {
        self.iter().map(|s| s.artel_str(ident_level)).join("\n\n")
    }
}

#[derive(Debug)]
pub enum ArtelStatement {
    LexicalDeclaration(ArtelLexicalDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ArtelClassDeclaration),
    TypeAliasDeclaration(ArtelTypeAliasDeclaration),
    InterfaceDeclaration(ArtelInterfaceDeclaration),
    EnumDeclaration(EnumDeclaration),
    ExportStatement(Box<ArtelStatement>),
    AmbientDeclaration(ArtelAmbientDeclaration),
    InternalModule(ArtelInternalModule),
    StatementBlock(ArtelStatements),
    Comment(String), // TODO
}

impl ArtelStr for ArtelStatement {
    fn artel_str(&self, ident_level: usize) -> String {
        match self {
            ArtelStatement::FunctionDeclaration(fd) => fd.artel_str(ident_level),
            ArtelStatement::EnumDeclaration(r#enum) => r#enum.artel_str(ident_level),
            ArtelStatement::InterfaceDeclaration(interface) => interface.artel_str(ident_level),
            ArtelStatement::ClassDeclaration(class) => class.artel_str(ident_level),
            ArtelStatement::ExportStatement(exprt) => format!(
                "{}внешнее\n{}",
                indent(ident_level),
                exprt.artel_str(ident_level + 2)
            ),
            ArtelStatement::AmbientDeclaration(decl) => decl.artel_str(0),
            ArtelStatement::TypeAliasDeclaration(typealias) => typealias.artel_str(ident_level),
            ArtelStatement::Comment(comment) => set_indent(comment, ident_level),
            ArtelStatement::LexicalDeclaration(decl) => decl.artel_str(ident_level),
            ArtelStatement::InternalModule(modul) => modul.artel_str(ident_level),
            _ => todo!("{self:?}"),
        }
    }
}
