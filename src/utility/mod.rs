use std::ops::{Rem, Sub};

pub fn padding<T>(offset: T, alignment: T) -> T
where
    T: Rem<Output = T> + Sub<Output = T> + Copy,
{
    (alignment - (offset % alignment)) % alignment
}
