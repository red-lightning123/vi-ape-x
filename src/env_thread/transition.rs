use super::State;

pub struct Transition {
    pub state: State,
    pub next_state: State,
    pub action: u8,
    pub reward: f64,
    pub terminated: bool,
}
