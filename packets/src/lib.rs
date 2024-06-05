use model::Params;
use replay_data::CompressedTransition;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub enum CoordinatorRequest {
    ActorConn,
    LearnerConn { service_addr: SocketAddr },
    ReplayConn { service_addr: SocketAddr },
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
    // The _size_marker member forces the serialized packet to have nonzero
    // size. ReplaySettings, the type of the settings member, is currently
    // a unit struct, so without _size_marker the packet may serialize to
    // nothing. We want to avoid such zero-sized packets because a deserializer
    // would have no way to tell whether they were actually transmitted through
    // its stream (as they don't occupy any bytes)
    pub _size_marker: u8,
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
