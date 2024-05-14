mod impls;
mod max_tree;
mod min_tree;
mod nodes;
mod priority_tree;
mod query_tree;
mod sum_tree;
mod traits;
mod tree;

use super::transition_saving;
use super::CompressedTransition;
use max_tree::MaxTree;
use min_tree::MinTree;
use nodes::{MaxNode, MinNode};
use priority_tree::{Priority, PriorityTree};
use sum_tree::SumTree;
pub use traits::{Infinity, NegativeInfinity, Zero};

pub struct PriorityCircBuffer<P: Copy + Zero + std::ops::Add<Output = P> + PartialOrd, V> {
    priorities: SumTree<P>,
    priorities_min: MinTree<MinNode<P>>,
    priorities_max: MaxTree<MaxNode<P>>,
    first_priority_leaf: usize,
    values: Vec<V>,
    max_size: usize,
    head: usize,
}
