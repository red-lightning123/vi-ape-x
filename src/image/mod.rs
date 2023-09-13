mod traits;
pub use traits::{ ImageRef, ImageOwned };
use traits::Zero;
mod image4;
pub use image4::{ Color4, ImageRef4, ImageOwned4 };
mod image2;
pub use image2::{ Color2, ImageRef2, ImageOwned2 };
