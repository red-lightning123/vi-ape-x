use super::{ImageOwned, ImageRef, Zero};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy)]
pub struct Color2(pub u8, pub u8);

impl Color2 {
    pub const fn new(a: u8, b: u8) -> Self {
        Self(a, b)
    }
}

pub struct WColor2(pub u32, pub u32);

impl WColor2 {
    pub const fn new(a: u32, b: u32) -> Self {
        Self(a, b)
    }
}

impl Zero for WColor2 {
    fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl std::ops::AddAssign for WColor2 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl std::ops::Div<u32> for WColor2 {
    type Output = Self;
    fn div(self, rhs: u32) -> Self {
        Self::new(self.0 / rhs, self.1 / rhs)
    }
}

impl From<Color2> for WColor2 {
    fn from(c: Color2) -> Self {
        Self::new(c.0.into(), c.1.into())
    }
}

impl TryFrom<WColor2> for Color2 {
    type Error = std::num::TryFromIntError;
    fn try_from(c: WColor2) -> Result<Self, Self::Error> {
        Ok(Self::new(c.0.try_into()?, c.1.try_into()?))
    }
}

pub struct ImageRef2<'a> {
    width: u32,
    height: u32,
    data: &'a [u8],
}

impl ImageRef2<'_> {
    pub fn new(width: u32, height: u32, data: &[u8]) -> ImageRef2 {
        ImageRef2 {
            width,
            height,
            data,
        }
    }
    pub fn data(&self) -> &[u8] {
        self.data
    }
    pub fn downscale_by_average(&self, new_width: u32, new_height: u32) -> ImageOwned2 {
        assert!(
            self.width() % new_width == 0 && self.height() % new_height != 0,
            "attempted to downscale to a problematic size"
        );
        let x_factor = self.width() / new_width;
        let y_factor = self.height() / new_height;
        let sample_size = x_factor * y_factor;
        let mut rescaled_image = ImageOwned2::zeroed(new_width, new_height);
        for y in 0..new_height {
            for x in 0..new_width {
                let mut sum = WColor2::zero();
                for y_inner in 0..y_factor {
                    for x_inner in 0..x_factor {
                        let original_color =
                            self.get_pixel_color(x * x_factor + x_inner, y * y_factor + y_inner);
                        sum += original_color.into();
                    }
                }
                rescaled_image.set_pixel_color(x, y, (sum / sample_size).try_into().unwrap());
            }
        }
        rescaled_image
    }
}

impl ImageRef for ImageRef2<'_> {
    type Owned = ImageOwned2;
    type Color = Color2;
    fn get_pixel_color(&self, x: u32, y: u32) -> Self::Color {
        let pixel = (2 * (x + y * self.width)) as usize;
        Self::Color::new(self.data[pixel], self.data[pixel + 1])
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageOwned2 {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl ImageOwned2 {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

impl ImageRef for ImageOwned2 {
    type Owned = Self;
    type Color = Color2;
    fn get_pixel_color(&self, x: u32, y: u32) -> Self::Color {
        self.as_ref().get_pixel_color(x, y)
    }
    fn width(&self) -> u32 {
        self.as_ref().width()
    }
    fn height(&self) -> u32 {
        self.as_ref().height()
    }
}

impl ImageOwned for ImageOwned2 {
    type Ref<'a> = ImageRef2<'a>;
    fn zeroed(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (2 * width * height) as usize],
        }
    }
    fn as_ref(&self) -> Self::Ref<'_> {
        ImageRef2::new(self.width, self.height, &self.data)
    }
    fn set_pixel_color(&mut self, x: u32, y: u32, color: Self::Color) {
        let pixel = (2 * (x + y * self.width)) as usize;
        self.data[pixel] = color.0;
        self.data[pixel + 1] = color.1;
    }
}
