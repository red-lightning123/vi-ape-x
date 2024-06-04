mod serializer_hack;

use coordinator_client::CoordinatorClient;
use local_ip_address::local_ip;
use packets::{ReplayRequest, ReplaySettings, SampleBatchErrorKind, SampleBatchResult};
use prompt::prompt_user_for_service_ip_addr;
use replay_memories::ReplayRing;
use serializer_hack::{SampleBatchReplySerializer, SampleBatchResultSerializer};
use std::net::{Ipv4Addr, TcpListener};

// # Rationale for enabling jemalloc
//
// The program is very demanding in terms of RAM usage, typically requiring
// several gigabytes of memory to run. This excess in memory is mostly due to
// the huge number of frames stored in the replay buffer.
// Unfortunately, as of the time of writing this comment, the program also
// exhibits significant heap fragmentation, further inflating the already high
// memory requirements. In fact, on a machine with 32 gigabytes of RAM, it
// often terminates with OOM.
// Fragmentation occurs when allocated objects are scattered in memory with many
// small gaps between them. Newly allocated objects will not be able fit in gaps
// that are too small for them, so in a sense the gaps become wasted memory.
// Fragmentation is a common symptom in long-running programs that make frequent
// allocations of varying sizes.
// There are ways to deal with fragmentation however. One common way to address
// it is by trying to recycle allocations via data structures such as memory
// arenas. Another strategy is to use a different memory allocator altogether.
// The jemalloc allocator explicitly aims toward fragmentation avoidance. It
// originated from FreeBSD's c library, and was previously used by rust on some
// platforms.
// Since jemalloc tries to avoid fragmentation, one might expect it to help with
// our fragmentation problem. Indeed, enabling jemalloc does seem to make the
// fragmentation negligible, and the program that previously exhausted 32 gigs
// of memory can now run with under 3 gigs (as of the time of writing)

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    let coordinator_ip_addr = prompt_user_for_service_ip_addr("coordinator");
    println!("coordinator ip addr set to {}...", coordinator_ip_addr);
    let coordinator_addr = (coordinator_ip_addr, ports::COORDINATOR).into();
    let coordinator_client = CoordinatorClient::new(coordinator_addr);
    let local_ip_addr = local_ip().unwrap();
    let local_addr = (local_ip_addr, ports::REPLAY).into();
    let settings = coordinator_client.replay_conn(local_addr);
    run(settings);
}

fn run(_settings: ReplaySettings) {
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
