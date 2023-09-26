use tensorflow::{ SessionRunArgs, Tensor, TensorType, FetchToken };

trait Output {
    fn fetch_from(session_run_args : &mut SessionRunArgs, fetch_token : FetchToken) -> Self;
}

impl<T : TensorType> Output for Tensor<T> {
    fn fetch_from(session_run_args : &mut SessionRunArgs, fetch_token : FetchToken) -> Self {
        session_run_args.fetch(fetch_token).unwrap()
    }
}

pub trait OutputTuple<const N : usize> {
    fn fetch_from(session_run_args : &mut SessionRunArgs, fetch_tokens : [FetchToken; N]) -> Self;
}

impl<T1 : Output> OutputTuple<1> for (T1,) {
    fn fetch_from(session_run_args : &mut SessionRunArgs, [fetch_token1] : [FetchToken; 1]) -> Self {
        (Output::fetch_from(session_run_args, fetch_token1),)
    }
}

impl<T1 : Output, T2 : Output> OutputTuple<2> for (T1, T2) {
    fn fetch_from(session_run_args : &mut SessionRunArgs, [fetch_token1, fetch_token2] : [FetchToken; 2]) -> Self {
        (Output::fetch_from(session_run_args, fetch_token1), Output::fetch_from(session_run_args, fetch_token2))
    }
}
