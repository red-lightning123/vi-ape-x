use tensorflow::Operation;
pub struct OutputLocation {
    op: Operation,
    index: i32,
}

impl OutputLocation {
    pub fn new(op: Operation, index: i32) -> Self {
        Self { op, index }
    }
    pub fn op(&self) -> &Operation {
        &self.op
    }
    pub fn index(&self) -> i32 {
        self.index
    }
}
