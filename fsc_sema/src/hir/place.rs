use super::LocalId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub local: LocalId,
}

impl Place {
    #[must_use]
    pub const fn new(local: LocalId) -> Self {
        Self { local }
    }
}
