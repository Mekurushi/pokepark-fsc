#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackSlot(pub i16);
#[derive(Debug)]
pub struct FrameLayout {
    param_count: i16,
    local_count: i16,
}

impl FrameLayout {
    pub fn new(param_count: i16, local_count: i16) -> Self {
        Self {
            param_count,
            local_count,
        }
    }
    pub fn local_count(&self) -> i16 {
        self.local_count
    }
    pub fn frame_size(&self) -> i16 {
        self.param_count + self.local_count
    }
}
