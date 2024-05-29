mod replay_prioritized;

use packets::ReplayRequest;
use replay_prioritized::ReplayPrioritized;
use std::net::TcpListener;

fn main() {
    const REPLAY_MAX_LEN: usize = 2_000_000;
    let socket = TcpListener::bind("localhost:43430").unwrap();
    let mut replay = ReplayPrioritized::with_max_size(REPLAY_MAX_LEN);
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = bincode::deserialize_from(&stream).unwrap();
        match request {
            ReplayRequest::ReleaseLock => todo!(),
            ReplayRequest::SampleBatch { batch_len } => {
                if replay.len() < batch_len {
                    todo!(
                        "reply to indicate that there aren't enough transitions for the batch_len"
                    );
                } else {
                    let _batch = replay.sample_batch(batch_len);
                    todo!("reply with the batch");
                }
            }
            ReplayRequest::InsertBatch { batch } => {
                for insertion in batch {
                    replay.add_transition_with_priority(insertion.transition, insertion.priority);
                }
            }
            ReplayRequest::UpdateBatchPriorities { batch: _ } => {
                panic!("clients should only update batch priorities after sampling a batch")
            }
        }
    }
}
