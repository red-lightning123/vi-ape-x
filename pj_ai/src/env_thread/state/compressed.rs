use std::rc::Rc;

use super::CompressedImageOwned2;
use super::GenericState;

pub type CompressedState = GenericState<Rc<CompressedImageOwned2>>;
