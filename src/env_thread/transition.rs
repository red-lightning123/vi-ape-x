use super::{CompressedState, State};

pub struct GenericTransition<S> {
    pub state: S,
    pub next_state: S,
    pub action: u8,
    pub reward: f64,
    pub terminated: bool,
}

pub type Transition = GenericTransition<State>;
pub type CompressedTransition = GenericTransition<CompressedState>;
