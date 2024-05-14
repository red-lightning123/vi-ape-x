use crate::ImageOwned2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedImageOwned2(ImageOwned2);

impl From<&CompressedImageOwned2> for ImageOwned2 {
    fn from(image: &CompressedImageOwned2) -> Self {
        image.0.clone()
    }
}

impl From<&ImageOwned2> for CompressedImageOwned2 {
    fn from(image: &ImageOwned2) -> Self {
        Self(image.clone())
    }
}
