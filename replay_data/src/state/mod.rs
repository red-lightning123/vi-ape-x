mod compressed;
mod generic;
mod normal;
mod saved;

pub use compressed::CompressedState;
use generic::GenericState;
pub use normal::State;
pub use saved::SavedState;

impl From<&CompressedState> for State {
    fn from(state: &CompressedState) -> Self {
        Self::from(state.frames().each_ref().map(|image| image.as_ref().into()))
    }
}
