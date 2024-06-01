use std::marker::PhantomData;

use super::filters::Filter;
use crate::env_thread::env::StateAccum;

#[derive(Clone)]
pub struct PipeFilterToAccum<F, A> {
    accum: A,
    _marker: PhantomData<F>,
}

impl<F, A> StateAccum for PipeFilterToAccum<F, A>
where
    F: Filter,
    A: StateAccum<Frame = <F as Filter>::Output>,
{
    type Frame = <F as Filter>::Input;
    type View = <A as StateAccum>::View;

    fn receive(&mut self, frame: Self::Frame) {
        self.accum.receive(F::call(frame));
    }

    fn view(&self) -> Self::View {
        self.accum.view()
    }
    fn reset_to_current(&mut self) {
        self.accum.reset_to_current()
    }
}

impl<F, A> From<<F as Filter>::Input> for PipeFilterToAccum<F, A>
where
    F: Filter,
    A: StateAccum<Frame = <F as Filter>::Output>,
{
    fn from(frame: <F as Filter>::Input) -> Self {
        Self {
            accum: A::from(F::call(frame)),
            _marker: PhantomData,
        }
    }
}
