mod replay_prioritized;
mod serializer_hack;

use packets::{ReplayRequest, SampleBatchErrorKind, SampleBatchResult};
use replay_prioritized::ReplayPrioritized;
use serializer_hack::{SampleBatchReplySerializer, SampleBatchResultSerializer};
use std::net::TcpListener;

fn main() {
    const REPLAY_MAX_LEN: usize = 2_000_000;
    let socket = TcpListener::bind("localhost:43430").unwrap();
    let mut replay = ReplayPrioritized::with_max_size(REPLAY_MAX_LEN);
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = bincode::deserialize_from(&stream).unwrap();
        match request {
            ReplayRequest::SampleBatch { batch_len } => {
                if replay.len() < batch_len {
                    let err = SampleBatchErrorKind::NotEnoughTransitions;
                    let result: SampleBatchResult = Err(err);
                    bincode::serialize_into(stream, &result).unwrap();
                } else {
                    let batch = replay.sample_batch(batch_len);
                    let reply = SampleBatchReplySerializer {
                        batch,
                        min_probability: replay.min_probability(),
                        replay_len: replay.len(),
                    };
                    let result: SampleBatchResultSerializer = Ok(reply);
                    bincode::serialize_into(&stream, &result).unwrap();

                    // Wait for the client to send a priority update request,
                    // then handle it
                    let request = bincode::deserialize_from(stream).unwrap();
                    match request {
                        ReplayRequest::UpdateBatchPriorities { batch } => {
                            let indices = batch
                                .iter()
                                .map(|priority_update| priority_update.index)
                                .collect::<Vec<_>>();
                            let priorities = batch
                                .iter()
                                .map(|priority_update| priority_update.priority)
                                .collect::<Vec<_>>();
                            replay.update_priorities(&indices, &priorities);
                        }
                        _ => panic!("client should update batch priorities immediately after sampling a batch"),
                    }
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
