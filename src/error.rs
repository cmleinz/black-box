#[derive(Debug, Clone, PartialEq)]
pub enum AddressError {
    Closed,
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("All addresses closed for actor")
    }
}

impl std::error::Error for AddressError {}
