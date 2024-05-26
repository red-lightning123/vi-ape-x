use crate::image::{ImageOwned, ImageOwned2, ImageRef};

use super::GenericState;

pub type State = GenericState<ImageOwned2>;

impl State {
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
