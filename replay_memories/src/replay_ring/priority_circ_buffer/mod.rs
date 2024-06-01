mod impls;
mod priority_tree;

pub use priority_tree::{Infinity, NegativeInfinity, Zero};
use priority_tree::{Priority, PriorityTree};

pub struct PriorityCircBuffer<P: Priority, V> {
    priorities: PriorityTree<P>,
    values: Vec<V>,
    max_size: usize,
    head: usize,
}
