mod basic_model;
pub mod traits;

pub use basic_model::BasicModel;

pub struct LearningStepInfo {
    pub loss: f32,
    pub average_q_val: f32,
}
