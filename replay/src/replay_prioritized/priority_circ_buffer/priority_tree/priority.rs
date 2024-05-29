use super::Zero;
use std::ops::{Add, Div, Mul, Sub, SubAssign};

pub trait Priority:
    Zero
    + Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + Div<Output = Self>
{
}
impl<P> Priority for P where
    P: Zero
        + Copy
        + PartialOrd
        + Add<Output = Self>
        + Sub<Output = Self>
        + SubAssign
        + Mul<Output = Self>
        + Div<Output = Self>
{
}
