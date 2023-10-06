mod traits;
use traits::Zero;
pub use traits::{ImageOwned, ImageRef};
mod image4;
pub use image4::{Color4, ImageOwned4, ImageRef4};
mod image2;
pub use image2::{Color2, ImageOwned2, ImageRef2};
