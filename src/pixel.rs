use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice;

type Point = (usize, usize);

/// A wrapper struct for a a one-dimensional vector that represents a square of values.
/// The wrapper makes it easier to index into the array (with `x` and `y` coordinates).
pub struct PixelSquare<T> {
    pixels: T,
    width: usize,
}

impl<T> PixelSquare<T> {
    pub fn new(pixels: T, width: usize) -> Self {
        PixelSquare { pixels, width }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.width
    }
}

impl<T> PixelSquare<&mut [T]> {
    /// Instantiates a new `PixelArrayMut` from a pointer to a C array
    pub unsafe fn from_raw_parts(data: *mut T, width: usize) -> Self {
        Self::new(slice::from_raw_parts_mut(data, width * width), width)
    }
}

impl<T, U: Deref<Target = [T]>> Index<usize> for PixelSquare<U> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.pixels[idx]
    }
}

impl<T, U: DerefMut<Target = [T]>> IndexMut<usize> for PixelSquare<U> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.pixels[idx]
    }
}

impl<T, U: Deref<Target = [T]>> Index<Point> for PixelSquare<U> {
    type Output = T;
    fn index(&self, (x, y): Point) -> &Self::Output {
        &self.pixels[y * self.width + x]
    }
}

impl<T, U: DerefMut<Target = [T]>> IndexMut<Point> for PixelSquare<U> {
    fn index_mut(&mut self, (x, y): Point) -> &mut Self::Output {
        &mut self.pixels[y * self.width + x]
    }
}
