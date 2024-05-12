mod arg;
mod arg_location;
mod each_ref;
mod output;
mod output_location;

use arg::ArgTuple;
use arg_location::ArgLocation;
use each_ref::each_ref;
use output::OutputTuple;
use output_location::OutputLocation;
use tensorflow::{Graph, SavedModelBundle, Session, SessionRunArgs};

pub struct TensorflowFn<const N: usize, const M: usize> {
    pub arg_locations: [ArgLocation; N],
    pub output_locations: [OutputLocation; M],
}

impl<const N: usize, const M: usize> TensorflowFn<N, M> {
    pub fn new(
        model_bundle: &SavedModelBundle,
        graph: &Graph,
        name: &str,
        arg_names: [&str; N],
        output_names: [(&str, i32); M],
    ) -> Self {
        let signature = model_bundle.meta_graph_def().get_signature(name).unwrap();
        let arg_infos = arg_names.map(|arg_name| signature.get_input(arg_name).unwrap());
        let arg_ops = arg_infos.map(|arg_info| {
            graph
                .operation_by_name_required(&arg_info.name().name)
                .unwrap()
        });
        let arg_locations = arg_ops.map(ArgLocation::new);
        let output_locations = output_names.map(|(name, index)| {
            let info = signature.get_output(name).unwrap();
            let op = graph.operation_by_name_required(&info.name().name).unwrap();
            OutputLocation::new(op, index)
        });
        Self {
            arg_locations,
            output_locations,
        }
    }
}

impl<const N: usize, const M: usize> TensorflowFn<N, M> {
    pub fn call<U: OutputTuple<M>>(&self, session: &Session, args: impl ArgTuple<N>) -> U {
        let mut session_run_args = SessionRunArgs::new();

        args.feed_to(&mut session_run_args, &self.arg_locations);

        let fetch_tokens = each_ref(&self.output_locations)
            .map(|location| session_run_args.request_fetch(location.op(), location.index()));

        session
            .run(&mut session_run_args)
            .expect("TensorflowFn couldn't run session");

        OutputTuple::fetch_from(&mut session_run_args, fetch_tokens)
    }
}
