use super::{State, Transition};
mod basic_model;
pub use basic_model::BasicModel;
mod prioritized_replay_wrapper;
pub mod traits;
pub use prioritized_replay_wrapper::PrioritizedReplayWrapper;
mod queue_replay_wrapper;
pub use queue_replay_wrapper::QueueReplayWrapper;
pub mod replay;

pub struct LearningStepInfo {
    pub loss: f32,
    pub average_q_val: f32,
}
