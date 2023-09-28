#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

/// Artel identifier, NewType of string for no apparent reason.
impl Identifier {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Identifier(name.into())
    }
}
