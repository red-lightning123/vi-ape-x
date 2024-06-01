mod compress;
mod pipe;
mod rc;

pub use compress::CompressFilter;
pub use pipe::FilterPipe;
pub use rc::RcFilter;

pub trait Filter {
    type Input;
    type Output;
    fn call(input: Self::Input) -> Self::Output;
}
