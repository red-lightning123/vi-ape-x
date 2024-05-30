use replay_data::CompressedTransition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LearnerRequest {
    GetParams,
}

#[derive(Serialize, Deserialize)]
pub struct Insertion {
    pub transition: CompressedTransition,
    pub priority: f64,
}

#[derive(Serialize, Deserialize)]
pub struct PriorityUpdate {
    pub index: usize,
    pub priority: f64,
}

#[derive(Serialize, Deserialize)]
pub enum ReplayRequest {
    ReleaseLock,
    SampleBatch { batch_len: usize },
    InsertBatch { batch: Vec<Insertion> },
    UpdateBatchPriorities { batch: Vec<PriorityUpdate> },
}

#[derive(Serialize, Deserialize)]
pub struct SampleBatchReply {
    pub batch: (Vec<usize>, Vec<f64>, Vec<CompressedTransition>),
    pub min_probability: f64,
    pub replay_len: usize,
}

#[derive(Serialize, Deserialize)]
pub enum SampleBatchErrorKind {
    NotEnoughTransitions,
}

pub type SampleBatchResult = Result<SampleBatchReply, SampleBatchErrorKind>;
