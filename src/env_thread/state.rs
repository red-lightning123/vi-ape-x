use crate::{ImageOwned, ImageOwned2, ImageRef};
use std::rc::Rc;

#[derive(Clone)]
pub struct State([Rc<ImageOwned2>; 4]);

impl State {
    pub fn frames(&self) -> &[Rc<ImageOwned2>; 4] {
        &self.0
    }
    fn frame_dims(&self) -> (u32, u32) {
        let frame = &self.frames()[0];
        (frame.width(), frame.height())
    }
    pub fn concat_frames(&self) -> ImageOwned2 {
        let (width, height) = self.frame_dims();
        let frames = self.frames();
        let mut concated_image = ImageOwned2::new(
            width,
            height * (frames.len() as u32),
            vec![0; 4 * (width * height * (frames.len() as u32)) as usize],
        );
        for (n_frame, frame) in frames.iter().enumerate() {
            for y in 0..height {
                for x in 0..width {
                    concated_image.set_pixel_color(
                        x,
                        y + (n_frame as u32) * height,
                        frame.get_pixel_color(x, y),
                    );
                }
            }
        }
        concated_image
    }
}

impl From<[Rc<ImageOwned2>; 4]> for State {
    fn from(frames: [Rc<ImageOwned2>; 4]) -> Self {
        Self(frames)
    }
}
