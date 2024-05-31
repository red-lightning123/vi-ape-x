pub trait StateAccum: From<Self::Frame> {
    type Frame;
    type View;
    fn receive(&mut self, frame: Self::Frame);
    fn view(&self) -> Self::View;
    fn reset_to_current(&mut self);
}
