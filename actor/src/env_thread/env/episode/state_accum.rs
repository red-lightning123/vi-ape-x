pub trait StateAccum {
    type Frame;
    type View;
    fn receive(&mut self, frame: Self::Frame);
    fn view(&self) -> Self::View;
    fn reset_to_current(&mut self);
    // We require an explicit from_frame method instead of relying on the
    // standard From trait because From has an identity blanket impl (which
    // simply returns self). That impl can sometimes conflict with custom impls
    // of From for StateAccums that accept generic frames, since the type system
    // may allow those frames to be of type Self. The main difference between
    // from_frame and the standard from method is that from_frame does not have
    // an identity blanket impl, avoiding the potential for conflict
    fn from_frame(frame: Self::Frame) -> Self;
}
