use super::ThreadId;
use std::path::PathBuf;

pub enum MasterThreadMessage {
    Done(ThreadId),
}

#[derive(Clone, Debug)]
pub enum MasterMessage {
    Save(PathBuf),
    Load(PathBuf),
    PrepareHold,
    Hold,
    Resume,
    Close,
}
