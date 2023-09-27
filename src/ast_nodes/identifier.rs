#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtelIdentifier(pub String);

/// Artel identifier, NewType of string for no apparent reason.
impl ArtelIdentifier {
    pub fn new<T: Into<String>>(name: T) -> Self {
        ArtelIdentifier(name.into())
    }
}
