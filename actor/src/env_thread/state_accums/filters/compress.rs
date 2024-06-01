use super::Filter;
use image::ImageOwned2;
use replay_data::CompressedImageOwned2;

#[derive(Clone)]
pub struct CompressFilter;

impl Filter for CompressFilter {
    type Input = ImageOwned2;
    type Output = CompressedImageOwned2;
    fn call(input: Self::Input) -> Self::Output {
        (&input).into()
    }
}
