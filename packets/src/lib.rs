mod compressed_image;
mod image;
mod state;
mod transition;

use transition::CompressedTransition;

pub enum LearnerRequest {
    GetParams,
}

pub struct Insertion {
    pub transition: CompressedTransition,
    pub priority: f64,
}

pub struct PriorityUpdate {
    pub index: usize,
    pub priority: f64,
}

pub enum ReplayRequest {
    ReleaseLock,
    SampleBatch { batch_len: usize },
    InsertBatch { batch: Vec<Insertion> },
    UpdateBatchPriorities { batch: Vec<PriorityUpdate> },
}
