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
