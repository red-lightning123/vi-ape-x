use crate::env_thread::env::StateAccum;
use replay_data::GenericState;
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Clone)]
pub struct FrameStack<Frame> {
    stack: VecDeque<Frame>,
}

impl<Frame> StateAccum for FrameStack<Frame>
where
    Frame: Clone + Debug,
{
    type Frame = Frame;
    type View = GenericState<Frame>;

    fn receive(&mut self, frame: Self::Frame) {
        self.stack.pop_front();
        self.stack.push_back(frame);
    }

    fn view(&self) -> Self::View {
        <[Frame; 4]>::try_from(Vec::from(self.stack.clone()))
            .expect("frame stack len should be 4")
            .into()
    }
    fn reset_to_current(&mut self) {
        let frame = self.stack.pop_back().unwrap();
        *self = Self::from_frame(frame);
    }

    fn from_frame(frame: Self::Frame) -> Self {
        Self {
            stack: VecDeque::from([frame.clone(), frame.clone(), frame.clone(), frame]),
        }
    }
}
