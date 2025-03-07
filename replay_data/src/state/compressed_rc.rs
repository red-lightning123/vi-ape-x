use super::GenericState;
use crate::compressed_image::CompressedImageOwned2;
use std::rc::Rc;

pub type CompressedRcState = GenericState<Rc<CompressedImageOwned2>>;
