use super::{Query, Tree, Zero};

#[derive(Debug)]
pub struct SumQuery;

impl<V: Copy + Zero + std::ops::Add<Output = V>> Query<V> for SumQuery {
    fn default() -> V {
        V::zero()
    }
    fn children_query(tree: &Tree<V>, node: usize) -> V {
        match tree.children(node) {
            (None, None) => V::zero(),
            (Some(left), None) => tree.value(left),
            (None, Some(right)) => tree.value(right),
            (Some(left), Some(right)) => tree.value(left) + tree.value(right),
        }
    }
}
