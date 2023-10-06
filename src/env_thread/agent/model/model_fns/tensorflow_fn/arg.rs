use super::ArgLocation;
use tensorflow::{SessionRunArgs, Tensor, TensorType};

trait Arg {
    fn feed_to<'a>(&'a self, session_run_args: &mut SessionRunArgs<'a>, location: &ArgLocation);
}

impl<T: TensorType> Arg for Tensor<T> {
    fn feed_to<'a>(&'a self, session_run_args: &mut SessionRunArgs<'a>, location: &ArgLocation) {
        session_run_args.add_feed(location.op(), 0, self);
    }
}

pub trait ArgTuple<const N: usize> {
    fn feed_to<'a>(
        &'a self,
        session_run_args: &mut SessionRunArgs<'a>,
        locations: &[ArgLocation; N],
    );
}

impl ArgTuple<0> for () {
    fn feed_to<'a>(&'a self, _session_run_args: &mut SessionRunArgs<'a>, []: &[ArgLocation; 0]) {}
}

impl<T1: Arg> ArgTuple<1> for (T1,) {
    fn feed_to<'a>(&'a self, session_run_args: &mut SessionRunArgs<'a>, [loc1]: &[ArgLocation; 1]) {
        self.0.feed_to(session_run_args, loc1);
    }
}

impl<T1: Arg, T2: Arg, T3: Arg, T4: Arg, T5: Arg> ArgTuple<5> for (T1, T2, T3, T4, T5) {
    fn feed_to<'a>(
        &'a self,
        session_run_args: &mut SessionRunArgs<'a>,
        [loc1, loc2, loc3, loc4, loc5]: &[ArgLocation; 5],
    ) {
        self.0.feed_to(session_run_args, loc1);
        self.1.feed_to(session_run_args, loc2);
        self.2.feed_to(session_run_args, loc3);
        self.3.feed_to(session_run_args, loc4);
        self.4.feed_to(session_run_args, loc5);
    }
}

impl<T1: Arg, T2: Arg, T3: Arg, T4: Arg, T5: Arg, T6: Arg, T7: Arg, T8: Arg, T9: Arg> ArgTuple<9>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
{
    fn feed_to<'a>(
        &'a self,
        session_run_args: &mut SessionRunArgs<'a>,
        [loc1, loc2, loc3, loc4, loc5, loc6, loc7, loc8, loc9]: &[ArgLocation; 9],
    ) {
        self.0.feed_to(session_run_args, loc1);
        self.1.feed_to(session_run_args, loc2);
        self.2.feed_to(session_run_args, loc3);
        self.3.feed_to(session_run_args, loc4);
        self.4.feed_to(session_run_args, loc5);
        self.5.feed_to(session_run_args, loc6);
        self.6.feed_to(session_run_args, loc7);
        self.7.feed_to(session_run_args, loc8);
        self.8.feed_to(session_run_args, loc9);
    }
}
