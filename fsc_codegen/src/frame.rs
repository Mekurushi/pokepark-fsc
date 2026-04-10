use crate::error::{CodegenError, CodegenResult};
use fsc_parse::ast::Param;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackSlot(pub i16);
pub struct FrameLayout {
    slots: HashMap<String, StackSlot>,
    param_count: i16,
    local_count: i16,
}

// TODO: moving that in typed AST/ HIR would be probably better
impl FrameLayout {
    pub fn from_params(params: &[Param]) -> Self {
        let slots = params
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), StackSlot(i as i16)))
            .collect();

        Self {
            slots,
            param_count: params.len() as i16,
            local_count: 0,
        }
    }
    // TODO: defined how temps should actually be used
    pub fn alloc_temp(&mut self) -> StackSlot {
        let slot = StackSlot(-(self.local_count + 1));
        self.slots.insert("tmp".to_string(), slot);
        if !self.slots.contains_key("tmp") {
            self.local_count += 1;
        }
        slot
    }
    pub fn resolve_temp(&mut self) -> CodegenResult<StackSlot> {
        self.slots
            .get("tmp")
            .copied()
            .ok_or_else(|| CodegenError::UndeclaredVariable("tmp".to_string()))
    }
    pub fn alloc_local(&mut self, name: &str) -> CodegenResult<StackSlot> {
        if self.slots.contains_key(name) {
            return Err(CodegenError::AlreadyDeclared(name.to_string()));
        }
        let slot = StackSlot(-(self.local_count + 1));
        self.slots.insert(name.to_string(), slot);
        self.local_count += 1;
        Ok(slot)
    }

    pub fn resolve(&self, name: &str) -> CodegenResult<StackSlot> {
        self.slots
            .get(name)
            .copied()
            .ok_or_else(|| CodegenError::UndeclaredVariable(name.to_string()))
    }

    pub fn local_count(&self) -> i16 {
        self.local_count
    }

    pub fn frame_size(&self) -> i16 {
        self.param_count + self.local_count
    }
}
