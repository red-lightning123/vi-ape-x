mod impls;

use super::transition_saving;
use priority_tree::{Priority, PriorityTree};

pub struct PriorityCircBuffer<P: Priority, V> {
    priorities: PriorityTree<P>,
    values: Vec<V>,
    max_size: usize,
    head: usize,
}
