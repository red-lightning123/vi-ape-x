use tensorflow::Operation;
pub struct ArgLocation {
    op : Operation
}

impl ArgLocation {
    pub fn new(op : Operation) -> ArgLocation {
        ArgLocation {
            op
        }
    }
    pub fn op(&self) -> &Operation {
        &self.op
    }
}

