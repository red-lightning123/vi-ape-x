mod compressed;
mod compressed_rc;
mod generic;
mod normal;
mod saved;

pub use compressed::CompressedState;
pub use compressed_rc::CompressedRcState;
use generic::GenericState;
pub use normal::State;
pub use saved::SavedState;

impl From<&CompressedState> for State {
    fn from(state: &CompressedState) -> Self {
        Self::from(state.frames().each_ref().map(|image| image.into()))
    }
}

impl From<&CompressedRcState> for State {
    fn from(state: &CompressedRcState) -> Self {
        Self::from(state.frames().each_ref().map(|image| image.as_ref().into()))
    }
}
