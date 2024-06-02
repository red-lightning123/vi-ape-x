mod serializer_hack;

use packets::{ReplayRequest, SampleBatchErrorKind, SampleBatchResult};
use replay_memories::ReplayRing;
use serializer_hack::{SampleBatchReplySerializer, SampleBatchResultSerializer};
use std::net::{Ipv4Addr, TcpListener};

fn main() {
    const REPLAY_MAX_LEN: usize = 3_000_000;
    const REPLAY_TRUNCATED_LEN: usize = 2_000_000;
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::REPLAY)).unwrap();
    let mut replay = ReplayRing::with_max_size(REPLAY_MAX_LEN);
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            ReplayRequest::Truncate => {
                replay.truncate(REPLAY_TRUNCATED_LEN);
            }
            ReplayRequest::SampleBatch { batch_len } => {
                const MIN_SAMPLING_REPLAY_SIZE: usize = 50_000;
                if replay.len() < MIN_SAMPLING_REPLAY_SIZE {
                    let err = SampleBatchErrorKind::NotEnoughTransitions;
                    let result: SampleBatchResult = Err(err);
                    tcp_io::serialize_into(stream, &result).unwrap();
                } else {
                    let batch = replay.sample_batch(batch_len);
                    let reply = SampleBatchReplySerializer {
                        batch,
                        min_probability: replay.min_probability(),
                        replay_len: replay.len(),
                    };
                    let result: SampleBatchResultSerializer = Ok(reply);
                    tcp_io::serialize_into(&stream, &result).unwrap();
                }
            }
            ReplayRequest::InsertBatch { batch } => {
                for insertion in batch {
                    replay.add_transition_with_priority(insertion.transition, insertion.priority);
                }
            }
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
        }
    }
}
