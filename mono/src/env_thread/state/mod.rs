mod compressed;
mod generic;
mod normal;
mod saved;

use super::image::CompressedImageOwned2;
pub use compressed::CompressedState;
use generic::GenericState;
pub use normal::State;
pub use saved::SavedState;

impl From<&CompressedState> for State {
    fn from(state: &CompressedState) -> Self {
        Self::from(state.frames().each_ref().map(|image| image.as_ref().into()))
    }
}
