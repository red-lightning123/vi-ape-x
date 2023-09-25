mod replay_memory;
pub use replay_memory::ReplayMemory;
mod replay_queue;
pub use replay_queue::ReplayQueue;
mod replay_prioritized;
pub use replay_prioritized::ReplayPrioritized;

type SavedTransition = ([usize; 4], [usize; 4], u8, f64, bool);

