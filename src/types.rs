/// Represents image dimensions (width, height) in pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn total_pixels(&self) -> usize {
        self.width * self.height
    }
}

impl From<(usize, usize)> for Dimensions {
    fn from((width, height): (usize, usize)) -> Self {
        Self { width, height }
    }
}

/// Represents a pixel coordinate (x, y)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
}

impl Pixel {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<(usize, usize)> for Pixel {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensions() {
        let dims = Dimensions::new(100, 200);
        assert_eq!(dims.width, 100);
        assert_eq!(dims.height, 200);
        assert_eq!(dims.total_pixels(), 20000);
    }
}
