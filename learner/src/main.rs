mod learner_schedule;

use coordinator_client::CoordinatorClient;
use learner_schedule::LearnerSchedule;
use local_ip_address::local_ip;
use model::traits::{ParamFetcher, TargetNet};
use model::BasicModel;
use packets::{GetParamsReply, LearnerRequest, LearnerSettings};
use prompt::prompt_user_for_service_ip_addr;
use replay_wrappers::RemoteReplayWrapper;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

fn spawn_batch_learner_thread(
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        const TARGET_UPDATE_INTERVAL_STEPS: u32 = 2_500;
        const BETA: f64 = 0.4;
        let mut schedule = LearnerSchedule::new(TARGET_UPDATE_INTERVAL_STEPS);
        loop {
            {
                let mut agent = agent.write().unwrap();
                agent.train_step(BETA);
                if schedule.is_time_to_update_target() {
                    agent.copy_control_to_target();
                }
            }
            schedule.step();
        }
    })
}

fn spawn_param_server_thread(
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::LEARNER)).unwrap();
        loop {
            let (stream, _source_addr) = socket.accept().unwrap();
            let request = tcp_io::deserialize_from(&stream).unwrap();
            match request {
                LearnerRequest::GetParams => {
                    let params = {
                        let agent = agent.read().unwrap();
                        agent.params()
                    };
                    let reply = GetParamsReply { params };
                    tcp_io::serialize_into(stream, &reply).unwrap();
                }
            }
        }
    })
}

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

fn enable_tf_memory_growth() {
    std::env::set_var("TF_FORCE_GPU_ALLOW_GROWTH", "true");
}

fn main() {
    // By default, tensorflow preallocates nearly all of the GPU memory
    // available. This behavior becomes a problem when multiple programs are
    // using it simultaneously, such as in a distributed reinforcement learning
    // agent, since it can easily result in GPU OOM. Fortunately, tensorflow has
    // an option to dynamically grow its allocated gpu memory, so we enable it
    // to circumvent the memory issue
    enable_tf_memory_growth();

    let coordinator_ip_addr = prompt_user_for_service_ip_addr("coordinator");
    println!("coordinator ip addr set to {}...", coordinator_ip_addr);
    let coordinator_addr = (coordinator_ip_addr, ports::COORDINATOR).into();
    let coordinator_client = CoordinatorClient::new(coordinator_addr);
    let local_ip_addr = local_ip().unwrap();
    let local_addr = (local_ip_addr, ports::LEARNER).into();
    let settings = coordinator_client.learner_conn(local_addr);
    run(settings);
}

fn run(settings: LearnerSettings) {
    const ALPHA: f64 = 0.6;
    let agent = Arc::new(RwLock::new(RemoteReplayWrapper::wrap(
        BasicModel::new(),
        settings.replay_server_addr,
        ALPHA,
    )));
    let batch_learner_thread = spawn_batch_learner_thread(Arc::clone(&agent));
    let param_server_thread = spawn_param_server_thread(agent);
    batch_learner_thread.join().unwrap();
    param_server_thread.join().unwrap();
}
