use super::{Infinity, Query, Tree};

#[derive(Debug)]
pub struct MinQuery;

impl<V: Copy + Infinity + PartialOrd> Query<V> for MinQuery {
    fn default() -> V {
        V::infinity()
    }
    fn children_query(tree: &Tree<V>, node: usize) -> V {
        match tree.children(node) {
            (None, None) => V::infinity(),
            (Some(left), None) => tree.value(left),
            (None, Some(right)) => tree.value(right),
            (Some(left), Some(right)) => {
                let left_value = tree.value(left);
                let right_value = tree.value(right);
                if left_value < right_value {
                    left_value
                } else {
                    right_value
                }
            }
        }
    }
}
