#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ActorError {
    Shutdown,
}
impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorError::Shutdown => f.write_str("Actor is already shutdown"),
        }
    }
}

impl std::error::Error for ActorError {}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AddressError {
    Closed,
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressError::Closed => f.write_str("All addresses closed for actor"),
        }
    }
}

impl std::error::Error for AddressError {}
