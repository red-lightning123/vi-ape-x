mod compressed;
mod generic;
mod normal;

use super::image::CompressedImageOwned2;
pub use compressed::CompressedState;
use generic::GenericState;
pub use normal::State;

impl From<&CompressedState> for State {
    fn from(state: &CompressedState) -> Self {
        Self::from(state.frames().clone().map(|image| image.as_ref().into()))
    }
}
