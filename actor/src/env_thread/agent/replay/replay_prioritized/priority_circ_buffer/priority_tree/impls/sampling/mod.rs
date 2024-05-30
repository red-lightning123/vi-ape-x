mod sample_sum_tree;

use super::SumTree;
use super::{Priority, PriorityTree};
use rand::distributions::{Distribution, Standard};
use rand::Rng;

impl<P: Priority> PriorityTree<P>
where
    Standard: Distribution<P>,
{
    pub fn sample_from_range<R>(&self, range_start: P, range_end: P, rng: &mut R) -> usize
    where
        R: Rng,
    {
        self.sum_tree.sample_from_range(range_start, range_end, rng) - self.first_leaf
    }
    pub fn sample<R>(&self, rng: &mut R) -> usize
    where
        R: Rng,
    {
        self.sum_tree.sample(rng) - self.first_leaf
    }
}
