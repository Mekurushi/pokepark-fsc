#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(u32);

impl LocalId {
    pub(crate) const fn new(index: u32) -> Self {
        Self(index)
    }
}
