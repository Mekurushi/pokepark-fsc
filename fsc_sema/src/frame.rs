use crate::error::{SemaError, SemaResult};
use fsc_parse::ast::{FuncDef, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackSlot(pub i16);
#[derive(Debug)]
pub struct FrameLayout {
    slots: HashMap<String, StackSlot>,

    param_count: i16,
    local_count: i16,
}

impl FrameLayout {
    pub fn resolve(&self, name: &str) -> SemaResult<StackSlot> {
        self.slots
            .get(name)
            .copied()
            .ok_or_else(|| SemaError::UndeclaredName(name.to_string()))
    }
    pub fn alloc_local(&mut self, name: &str) -> SemaResult<StackSlot> {
        if self.slots.contains_key(name) {
            return Err(SemaError::DuplicateDeclaration(name.to_string()));
        }
        let slot = StackSlot(-(self.local_count + 1));
        self.slots.insert(name.to_string(), slot);
        self.local_count += 1;
        Ok(slot)
    }
    pub fn local_count(&self) -> i16 {
        self.local_count
    }
    pub fn frame_size(&self) -> i16 {
        self.param_count + self.local_count
    }
}

pub fn build_frame(func: &FuncDef) -> SemaResult<FrameLayout> {
    let mut slots: HashMap<String, StackSlot> = HashMap::new();
    let param_count = func.params.len() as i16;

    for (i, param) in func.params.iter().enumerate() {
        if slots.contains_key(&param.name) {
            return Err(SemaError::DuplicateDeclaration(param.name.clone()));
        }
        slots.insert(param.name.clone(), StackSlot(i as i16));
    }

    let mut local_count: i16 = 0;
    collect_locals(&func.body, &mut slots, &mut local_count)?;

    Ok(FrameLayout {
        slots,
        param_count,
        local_count,
    })
}

fn collect_locals(
    stmts: &Vec<Stmt>,
    slots: &mut HashMap<String, StackSlot>,
    local_count: &mut i16,
) -> SemaResult<()> {
    for stmt in stmts {
        if let Stmt::VarDecl { name, .. } = stmt {
            if slots.contains_key(name) {
                return Err(SemaError::DuplicateDeclaration(name.to_string()));
            }
            *local_count += 1;
            let slot = StackSlot(-(*local_count));
            slots.insert(name.to_string(), slot);
        }
    }
    Ok(())
}
