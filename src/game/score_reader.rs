use crate::Color4;
use crate::{ImageOwned, ImageRef, ImageRef4};
use std::collections::HashMap;

pub enum ScoreReaderError {
    InvalidColor,
    InvalidDigitShape,
}

pub struct ScoreReader {
    symbol_to_digit_class: HashMap<u64, u8>,
}

impl ScoreReader {
    pub fn new() -> Self {
        let mut symbol_to_digit_class = HashMap::new();
        let reference_str = include_str!("digits");
        let mut reference_str_it = reference_str.split_whitespace();
        // 10 classes for 10 digits, and 1 more for an empty digit
        for _ in 0..11 {
            let name = reference_str_it.next().unwrap();
            let class = if name == "None" {
                10
            } else {
                name.parse::<u8>().unwrap()
            };
            let mut symbol: u64 = 0;
            for _ in 0..8 {
                let line = reference_str_it.next().unwrap();
                for c in line.chars() {
                    symbol <<= 1;
                    if c == '.' {
                    } else if c == '#' {
                        symbol += 1;
                    } else {
                        panic!("ScoreReader::new encountered unrecognized character");
                    }
                }
            }
            symbol_to_digit_class.insert(symbol, class);
        }
        Self {
            symbol_to_digit_class,
        }
    }
    pub fn read_score(&self, image_ref: ImageRef4) -> Result<u32, ScoreReaderError> {
        const N_DIGITS_TO_READ: u32 = 6;
        let mut score = 0;
        for i in 0..N_DIGITS_TO_READ {
            let next_digit = self.read_score_digit(&image_ref, i)?;
            if let Some(next_digit) = next_digit {
                score *= 10;
                score += u32::from(next_digit);
            } else {
                break;
            }
        }
        Ok(score)
    }
    fn read_score_digit(
        &self,
        image_ref: &ImageRef4,
        pos: u32,
    ) -> Result<Option<u8>, ScoreReaderError> {
        let image_cropped =
            image_ref.crop(1680 + 32 * pos, 1680 + 32 * pos + 32 - 8, 142, 142 + 32);
        let image_downscaled = image_cropped
            .downscale_by_sample(image_cropped.width() / 4, image_cropped.height() / 4);
        self.classify_digit_image(image_downscaled.as_ref())
    }
    fn classify_digit_image(&self, image_ref: ImageRef4) -> Result<Option<u8>, ScoreReaderError> {
        let symbol = Self::digit_image_to_symbol(&image_ref)?;
        let class = if let Some(class) = self.symbol_to_digit_class.get(&symbol) {
            *class
        } else {
            return Err(ScoreReaderError::InvalidDigitShape);
        };
        if class == 10 {
            Ok(None)
        } else {
            Ok(Some(class))
        }
    }
    fn digit_image_to_symbol(image_ref: &ImageRef4) -> Result<u64, ScoreReaderError> {
        let mut symbol = 0;
        for y in 0..image_ref.height() {
            for x in 0..image_ref.width() {
                symbol <<= 1;
                let color = image_ref.get_pixel_color(x, y);
                const DIGIT_COLOR_OFF: Color4 = Color4::new(47, 47, 47, 255);
                const DIGIT_COLOR_ON: Color4 = Color4::new(255, 255, 255, 255);
                if color == DIGIT_COLOR_OFF {
                } else if color == DIGIT_COLOR_ON {
                    symbol += 1;
                } else {
                    return Err(ScoreReaderError::InvalidColor);
                }
            }
        }
        Ok(symbol)
    }
}
