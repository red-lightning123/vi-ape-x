use super::Filter;
use std::{marker::PhantomData, rc::Rc};

#[derive(Clone)]
pub struct RcFilter<Input> {
    _marker: PhantomData<Input>,
}

impl<Input> Filter for RcFilter<Input> {
    type Input = Input;
    type Output = Rc<Input>;
    fn call(input: Self::Input) -> Self::Output {
        Rc::new(input)
    }
}
