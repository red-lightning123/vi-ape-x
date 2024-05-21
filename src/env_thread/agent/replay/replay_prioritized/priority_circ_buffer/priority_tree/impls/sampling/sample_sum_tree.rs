use super::Priority;
use super::SumTree;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

impl<P: Priority> SumTree<P> {
    fn sample_by_priority_sum_from_left(&self, mut priority_sum_from_left: P) -> usize {
        let mut node = self.root();
        loop {
            match self.children(node) {
                (None, None) => return node,
                (Some(left), None) => node = left,
                (None, Some(right)) => node = right,
                (Some(left), Some(right)) => {
                    let priority_left = self.value(left);
                    let chose_left = priority_sum_from_left < priority_left;
                    node = if chose_left {
                        left
                    } else {
                        priority_sum_from_left -= priority_left;
                        right
                    };
                }
            }
        }
    }
}

impl<P: Priority> SumTree<P>
where
    Standard: Distribution<P>,
{
    pub fn sample_from_range<R>(&self, range_start: P, range_end: P, rng: &mut R) -> usize
    where
        R: Rng,
    {
        let priority_total = self.value(self.root());
        let point_chosen = range_start + rng.gen::<P>() * (range_end - range_start);
        let point_scaled = point_chosen * priority_total;
        self.sample_by_priority_sum_from_left(point_scaled)
    }
    pub fn sample<R>(&self, rng: &mut R) -> usize
    where
        R: Rng,
    {
        let priority_total = self.value(self.root());
        self.sample_by_priority_sum_from_left(rng.gen::<P>() * priority_total)
    }
}
