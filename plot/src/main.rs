mod plot_set;

use coordinator_client::CoordinatorClient;
use local_ip_address::local_ip;
use packets::{PlotRequest, PlotSettings};
use plot_set::PlotSet;
use prompt::prompt_user_for_service_ip_addr;
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
    let local_addr = (local_ip_addr, ports::PLOT).into();
    let settings = coordinator_client.plot_conn(local_addr);
    run(settings);
}

fn run(_settings: PlotSettings) {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::PLOT)).unwrap();
    let mut plot_set = PlotSet::new("progress");
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            PlotRequest { kind, batch } => {
                for datum in batch {
                    plot_set.add_datum(kind, datum);
                }
            }
        }
    }
}
