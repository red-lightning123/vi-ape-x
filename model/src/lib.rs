mod basic_model;
pub mod traits;

pub use basic_model::BasicModel;
use serde::{Deserialize, Serialize};

pub struct LearningStepInfo {
    pub loss: f32,
    pub average_q_val: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Params(Vec<String>);
