use tensorflow::Operation;
pub struct ArgLocation {
    op: Operation,
}

impl ArgLocation {
    pub fn new(op: Operation) -> Self {
        Self { op }
    }
    pub fn op(&self) -> &Operation {
        &self.op
    }
}
