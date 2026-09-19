use fsc_parse::ast::Ty;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
}

impl ConfigValue {
    pub(crate) fn ty(&self) -> Ty {
        match self {
            Self::Int(_) => Ty::Int,
            Self::Float(_) => Ty::Float,
            Self::Bool(_) => Ty::Bool,
            Self::String(_) => Ty::Str,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfigValues {
    pub(crate) values: BTreeMap<String, ConfigValue>,
}

impl ConfigValues {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: impl Into<String>, value: ConfigValue) -> Option<ConfigValue> {
        self.values.insert(name.into(), value)
    }
}
