// TODO: doesn't feel right; check if it should be restructured
pub struct LabelCtx {
    label_counter: u32,
}

impl LabelCtx {
    pub fn new() -> Self {
        Self { label_counter: 0 }
    }

    pub fn fresh_label(&mut self, prefix: &str) -> String {
        let n = self.label_counter;
        self.label_counter += 1;
        format!("{prefix}_{n}")
    }
}
