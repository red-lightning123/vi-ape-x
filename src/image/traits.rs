pub trait Zero {
    fn zero() -> Self;
}

pub trait ImageRef {
    type Owned : ImageOwned<Color=Self::Color>;
    type Color : PartialEq + Copy;
    fn get_pixel_color(&self, x : u32, y : u32) -> Self::Color;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn crop(&self, start_x : u32, end_x : u32, start_y : u32, end_y : u32) -> Self::Owned {
        let cropped_width = end_x - start_x;
        let cropped_height = end_y - start_y;
        let mut cropped_image = Self::Owned::zeroed(cropped_width, cropped_height);
        for y in start_y..end_y {
            for x in start_x..end_x {
                let original_color = self.get_pixel_color(x, y);
                cropped_image.set_pixel_color(x - start_x, y - start_y, original_color);
            }
        }
        cropped_image
    }
    fn downscale_by_sample(&self, new_width : u32, new_height : u32) -> Self::Owned {
        assert!(self.width() % new_width == 0 && self.height() % new_height == 0, "attempted to downscale to a problematic size");
        let x_factor = self.width() / new_width;
        let y_factor = self.height() / new_height;
        let mut rescaled_image = Self::Owned::zeroed(new_width, new_height);
        for y in 0..new_height {
            for x in 0..new_width {
                let original_color = self.get_pixel_color(x * x_factor, y * y_factor);
                rescaled_image.set_pixel_color(x, y, original_color);
            }
        }
        rescaled_image
    }
}

pub trait ImageOwned
where Self: ImageRef {
    type Ref<'a> : ImageRef where Self: 'a;
    fn zeroed(width : u32, height : u32) -> Self;
    fn set_pixel_color(&mut self, x : u32, y : u32, color : Self::Color);
    fn as_ref(&self) -> Self::Ref<'_>;
    fn map_color<F : Fn(Self::Color) -> Self::Color>(&mut self, f : F) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let original_color = self.get_pixel_color(x, y);
                self.set_pixel_color(x, y, f(original_color));
            }
        }
    }
    fn replace_color_for<F : Fn(Self::Color) -> bool>(&mut self, f : F, new_color : Self::Color) {
        self.map_color(|c| if f(c) { new_color } else { c } );
    }
    fn replace_color(&mut self, old_color : Self::Color, new_color : Self::Color) {
        self.replace_color_for(|c| c == old_color, new_color);
    }
    fn replace_area_color(&mut self, (start_x, end_x) : (u32, u32), (start_y, end_y) : (u32, u32), new_color : Self::Color) {
        for y in start_y..end_y {
            for x in start_x..end_x {
                self.set_pixel_color(x, y, new_color);
            }
        }
    }
}
