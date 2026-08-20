#[derive(Debug, PartialEq, Eq)]
pub enum PatchError {
    UnalignedCode,
    InvalidEntryOffset(u32),
    JumpOutOfRange { entry: u32, target: u32 },
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnalignedCode => f.write_str("FSB code and appended code must be 4-byte aligned"),
            Self::InvalidEntryOffset(offset) => {
                write!(
                    f,
                    "function entry offset {offset:#x} is outside the FSB code section"
                )
            }
            Self::JumpOutOfRange { entry, target } => {
                write!(f, "entry jump is out of range ({entry:#x} -> {target:#x})")
            }
        }
    }
}

impl std::error::Error for PatchError {}

#[derive(Debug, PartialEq, Eq)]
pub enum PatchFailure {
    NotImplemented,
}

impl std::fmt::Display for PatchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => f.write_str("patching is not implemented yet"),
        }
    }
}

impl std::error::Error for PatchFailure {}

#[derive(Debug)]
pub struct SymbolTableParseError {
    pub(crate) source: toml::de::Error,
}

impl std::fmt::Display for SymbolTableParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}

impl std::error::Error for SymbolTableParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
