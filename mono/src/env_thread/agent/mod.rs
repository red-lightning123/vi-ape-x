mod basic_model;
mod prioritized_replay_wrapper;
mod queue_replay_wrapper;
pub mod replay;
pub mod traits;

pub use basic_model::BasicModel;
pub use prioritized_replay_wrapper::PrioritizedReplayWrapper;
pub use queue_replay_wrapper::QueueReplayWrapper;

pub struct LearningStepInfo {
    pub loss: f32,
    pub average_q_val: f32,
}
