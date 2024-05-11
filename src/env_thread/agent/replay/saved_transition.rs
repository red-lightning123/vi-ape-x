use serde::{Deserialize, Serialize};
// Similiar to Transition, but state and next_state are stored as indices into
// an external frame array
#[derive(Serialize, Deserialize)]
pub struct SavedTransition {
    pub state_frame_indices: [usize; 4],
    pub next_state_frame_indices: [usize; 4],
    pub action: u8,
    pub reward: f64,
    pub terminated: bool,
}
