pub use super::*;

#[derive(Debug, Clone)]
pub struct ArtelExpression(pub String);

impl ArtelStr for ArtelExpression {
    fn artel_str(&self, _ident_level: usize) -> String {
        match self.0.as_str() {
            "false" => "нет".to_owned(),
            "true" => "да".to_owned(),
            _ => self.0.clone(),
        }
    }
}
