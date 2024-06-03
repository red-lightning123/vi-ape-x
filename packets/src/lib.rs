use model::Params;
use replay_data::CompressedTransition;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub enum CoordinatorRequest {
    ActorConn,
    LearnerConn,
    ReplayConn,
    Start,
}

#[derive(Serialize, Deserialize)]
pub struct ActorSettings {
    pub learner_addr: SocketAddr,
    pub replay_server_addr: SocketAddr,
    pub eps: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ActorConnReply {
    pub settings: ActorSettings,
}

#[derive(Serialize, Deserialize)]
pub struct LearnerSettings {
    pub replay_server_addr: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub struct LearnerConnReply {
    pub settings: LearnerSettings,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaySettings;

#[derive(Serialize, Deserialize)]
pub struct ReplayConnReply {
    pub settings: ReplaySettings,
}

#[derive(Serialize, Deserialize)]
pub enum LearnerRequest {
    GetParams,
}

#[derive(Serialize, Deserialize)]
pub struct GetParamsReply {
    pub params: Params,
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
    Truncate,
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
