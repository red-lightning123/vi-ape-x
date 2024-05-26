use image::{ImageOwned, ImageOwned2, ImageRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedImageOwned2 {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl From<&CompressedImageOwned2> for ImageOwned2 {
    fn from(image: &CompressedImageOwned2) -> Self {
        Self::new(
            image.width,
            image.height,
            zstd::decode_all(image.data.as_slice()).unwrap(),
        )
    }
}

impl From<&ImageOwned2> for CompressedImageOwned2 {
    fn from(image: &ImageOwned2) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
            data: zstd::encode_all(image.as_ref().data(), 0).unwrap(),
        }
    }
}
