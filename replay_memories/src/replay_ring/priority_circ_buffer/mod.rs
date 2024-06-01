mod impls;

use priority_tree::{Priority, PriorityTree};

pub struct PriorityCircBuffer<P: Priority, V> {
    priorities: PriorityTree<P>,
    values: Vec<Option<V>>,
    max_size: usize,
    head: usize,
    tail: usize,
}
