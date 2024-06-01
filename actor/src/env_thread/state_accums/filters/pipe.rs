use super::Filter;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct FilterPipe<F1, F2> {
    _marker: PhantomData<(F1, F2)>,
}

impl<F1, F2> Filter for FilterPipe<F1, F2>
where
    F1: Filter,
    F2: Filter<Input = F1::Output>,
{
    type Input = <F1 as Filter>::Input;
    type Output = <F2 as Filter>::Output;
    fn call(input: Self::Input) -> Self::Output {
        F2::call(F1::call(input))
    }
}
