mod impls;
mod max_tree;
mod min_tree;
mod nodes;
mod priority;
mod query_tree;
mod sum_tree;
mod traits;
mod tree;

use max_tree::MaxTree;
use min_tree::MinTree;
use nodes::{MaxNode, MinNode};
pub use priority::Priority;
use serde::{Deserialize, Serialize};
use sum_tree::SumTree;
use traits::{Infinity, NegativeInfinity, Zero};

#[derive(Serialize, Deserialize)]
pub struct PriorityTree<P: Priority> {
    sum_tree: SumTree<P>,
    min_tree: MinTree<MinNode<P>>,
    max_tree: MaxTree<MaxNode<P>>,
    first_leaf: usize,
}

impl Zero for f64 {
    fn zero() -> Self {
        0.0
    }
}
