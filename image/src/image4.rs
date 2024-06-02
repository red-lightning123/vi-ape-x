use super::{ImageOwned, ImageRef, Zero};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Color4(pub u8, pub u8, pub u8, pub u8);

impl Color4 {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self(a, b, c, d)
    }
}

pub struct WColor4(pub u32, pub u32, pub u32, pub u32);

impl WColor4 {
    pub const fn new(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self(a, b, c, d)
    }
}

impl Zero for WColor4 {
    fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl std::ops::AddAssign for WColor4 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
        self.3 += rhs.3;
    }
}

impl std::ops::Div<u32> for WColor4 {
    type Output = Self;
    fn div(self, rhs: u32) -> Self {
        Self::new(self.0 / rhs, self.1 / rhs, self.2 / rhs, self.3 / rhs)
    }
}

impl From<Color4> for WColor4 {
    fn from(c: Color4) -> Self {
        Self::new(c.0.into(), c.1.into(), c.2.into(), c.3.into())
    }
}

impl TryFrom<WColor4> for Color4 {
    type Error = std::num::TryFromIntError;
    fn try_from(c: WColor4) -> Result<Self, Self::Error> {
        Ok(Self::new(
            c.0.try_into()?,
            c.1.try_into()?,
            c.2.try_into()?,
            c.3.try_into()?,
        ))
    }
}

pub struct ImageRef4<'a> {
    width: u32,
    height: u32,
    data: &'a [u8],
}

impl ImageRef4<'_> {
    pub fn new(width: u32, height: u32, data: &[u8]) -> ImageRef4 {
        ImageRef4 {
            width,
            height,
            data,
        }
    }
    pub fn data(&self) -> &[u8] {
        self.data
    }
    pub fn downscale_by_average(&self, new_width: u32, new_height: u32) -> ImageOwned4 {
        assert!(
            self.width() % new_width == 0 && self.height() % new_height == 0,
            "attempted to downscale to a problematic size"
        );
        let x_factor = self.width() / new_width;
        let y_factor = self.height() / new_height;
        let sample_size = x_factor * y_factor;
        let mut rescaled_image = ImageOwned4::zeroed(new_width, new_height);
        for y in 0..new_height {
            for x in 0..new_width {
                let mut sum = WColor4::zero();
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

impl ImageRef for ImageRef4<'_> {
    type Owned = ImageOwned4;
    type Color = Color4;
    fn get_pixel_color(&self, x: u32, y: u32) -> Self::Color {
        let pixel = (4 * (x + y * self.width)) as usize;
        Self::Color::new(
            self.data[pixel],
            self.data[pixel + 1],
            self.data[pixel + 2],
            self.data[pixel + 3],
        )
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}

pub struct ImageOwned4 {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl ImageOwned4 {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

impl ImageRef for ImageOwned4 {
    type Owned = Self;
    type Color = Color4;
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

impl ImageOwned for ImageOwned4 {
    type Ref<'a> = ImageRef4<'a>;
    fn zeroed(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (4 * width * height) as usize],
        }
    }
    fn as_ref(&self) -> Self::Ref<'_> {
        ImageRef4::new(self.width, self.height, &self.data)
    }
    fn set_pixel_color(&mut self, x: u32, y: u32, color: Self::Color) {
        let pixel = (4 * (x + y * self.width)) as usize;
        self.data[pixel] = color.0;
        self.data[pixel + 1] = color.1;
        self.data[pixel + 2] = color.2;
        self.data[pixel + 3] = color.3;
    }
}
