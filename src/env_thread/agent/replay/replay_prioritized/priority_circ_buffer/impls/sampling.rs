use super::PriorityCircBuffer;
use super::{Priority, PriorityTree};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::ops::Div;

impl<P: Priority, V> PriorityCircBuffer<P, V>
where
    Standard: Distribution<<P as Div>::Output>,
    <P as Div>::Output: PartialOrd,
{
    pub fn sample_from_range<R>(&self, range_start: P, range_end: P, rng: &mut R) -> (usize, P, &V)
    where
        R: Rng,
    {
        let index = self
            .priorities
            .sample_from_range(range_start, range_end, rng);
        let value_index = index - self.first_priority_leaf;
        let priority = self.priorities.value(index);
        let value = &self.values[value_index];
        (index, priority, value)
    }
    pub fn sample<R>(&self, rng: &mut R) -> (usize, P, &V)
    where
        R: Rng,
    {
        // TODO: the actual tree node index is an implementation
        // detail so it should be encapsulated in a wrapper type
        let index = self.priorities.sample(rng);
        let value_index = index - self.first_priority_leaf;
        let priority = self.priorities.value(index);
        let value = &self.values[value_index];
        (index, priority, value)
    }
}
