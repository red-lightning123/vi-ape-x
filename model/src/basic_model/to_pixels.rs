use std::rc::Rc;

use image::{ImageOwned, ImageOwned2, ImageRef2};
use replay_data::{CompressedImageOwned2, GenericState};

fn extract_planes(frame: &ImageRef2) -> (Vec<u8>, Vec<u8>) {
    frame.data().chunks(2).map(|a| (a[0], a[1])).unzip()
}

pub trait ToPixels {
    fn to_pixels(&self) -> Vec<u8>;
}

impl<'a> ToPixels for ImageRef2<'a> {
    fn to_pixels(&self) -> Vec<u8> {
        let (plane_0, plane_1) = extract_planes(self);
        [plane_0, plane_1].concat()
    }
}

impl ToPixels for ImageOwned2 {
    fn to_pixels(&self) -> Vec<u8> {
        self.as_ref().to_pixels()
    }
}

impl ToPixels for CompressedImageOwned2 {
    fn to_pixels(&self) -> Vec<u8> {
        let self_decompressed: ImageOwned2 = self.into();
        self_decompressed.to_pixels()
    }
}

impl<I> ToPixels for Rc<I>
where
    I: ToPixels,
{
    fn to_pixels(&self) -> Vec<u8> {
        self.as_ref().to_pixels()
    }
}

impl<I> ToPixels for GenericState<I>
where
    I: ToPixels,
{
    fn to_pixels(&self) -> Vec<u8> {
        self.frames()
            .iter()
            .map(ToPixels::to_pixels)
            .collect::<Vec<_>>()
            .concat()
    }
}
