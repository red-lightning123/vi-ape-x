use super::{NegativeInfinity, Query, Tree};

#[derive(Debug)]
pub struct MaxQuery;

impl<V: Copy + NegativeInfinity + PartialOrd> Query<V> for MaxQuery {
    fn default() -> V {
        V::negative_infinity()
    }
    fn children_query(tree: &Tree<V>, node: usize) -> V {
        match tree.children(node) {
            (None, None) => V::negative_infinity(),
            (Some(left), None) => tree.value(left),
            (None, Some(right)) => tree.value(right),
            (Some(left), Some(right)) => {
                let left_value = tree.value(left);
                let right_value = tree.value(right);
                if left_value < right_value {
                    right_value
                } else {
                    left_value
                }
            }
        }
    }
}
