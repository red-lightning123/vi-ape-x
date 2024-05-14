#[derive(Clone)]
pub struct GenericState<F>([F; 4]);

impl<F> GenericState<F> {
    pub fn frames(&self) -> &[F; 4] {
        &self.0
    }
}

impl<F> From<[F; 4]> for GenericState<F> {
    fn from(frames: [F; 4]) -> Self {
        Self(frames)
    }
}
