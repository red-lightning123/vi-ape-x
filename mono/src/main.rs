#![allow(dead_code, unused_imports)]
mod env_thread;
mod game;
mod game_interface;
mod game_thread;
mod human_interface;
mod keycodes;
mod master_thread;
mod plot_thread;
mod ui_thread;
mod x11_utils;

use env_thread::{spawn_env_thread, EnvThreadMessage};
use game::Game;
use game_interface::{GameInterface, GameKey, KeyEventKind};
use game_thread::{spawn_game_thread, GameThreadMessage};
use human_interface::HumanInterface;
use master_thread::{spawn_master_thread, MasterMessage, MasterThreadMessage, ThreadId};
use plot_thread::{spawn_plot_thread, PlotThreadMessage, PlotType};
use ui_thread::{spawn_ui_thread, UiThreadMessage};
use x11_utils::{choose_matching_fbconfigs, GlxContext, Window, X11Display};

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
    let master_thread = spawn_master_thread();
    master_thread.join().unwrap();
}
