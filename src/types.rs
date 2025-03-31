use num::Complex;

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

/// Represents a rectangular region in the complex plane
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexRegion {
    pub upper_left: Complex<f64>,
    pub lower_right: Complex<f64>,
}

impl ComplexRegion {
    pub fn new(upper_left: Complex<f64>, lower_right: Complex<f64>) -> Self {
        Self {
            upper_left,
            lower_right,
        }
    }

    /// Calculate the width and height of this region
    pub fn dimensions(&self) -> (f64, f64) {
        let width = self.lower_right.re - self.upper_left.re;
        let height = self.upper_left.im - self.lower_right.im;
        (width, height)
    }

    /// Convert a pixel coordinate to a point in the complex plane
    pub fn pixel_to_point(&self, bounds: Dimensions, pixel: Pixel) -> Complex<f64> {
        let (width, height) = self.dimensions();

        Complex {
            re: self.upper_left.re + pixel.x as f64 * width / bounds.width as f64,
            im: self.upper_left.im - pixel.y as f64 * height / bounds.height as f64,
        }
    }
}

/// Calculate the corners of a square region in the complex plane
///
/// Given a magnitude and center point, finds the upper-left and lower-right corners
/// of the containing square in the complex plane.
///
/// # Arguments
///
/// * `magnitude` - The width and height of the square
/// * `center` - The (real, imaginary) coordinates of the center point
///
/// # Returns
///
/// Returns a `ComplexRegion` representing the square
pub fn calculate_region(magnitude: f64, center: (f64, f64)) -> ComplexRegion {
    let half_mag = magnitude / 2.0;
    let (center_x, center_y) = center;
    
    ComplexRegion::new(
        Complex::new(center_x - half_mag, center_y + half_mag),
        Complex::new(center_x + half_mag, center_y - half_mag),
    )
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

    #[test]
    fn test_calculate_region() {
        let region = calculate_region(3.0, (0.0, 0.0));
        assert_eq!(region.upper_left, Complex::new(-1.5, 1.5));
        assert_eq!(region.lower_right, Complex::new(1.5, -1.5));

        let region = calculate_region(2.0, (-0.5, 0.0));
        assert_eq!(region.upper_left, Complex::new(-1.5, 1.0));
        assert_eq!(region.lower_right, Complex::new(0.5, -1.0));
    }

    #[test]
    fn test_pixel_to_point() {
        let bounds = Dimensions::new(100, 200);
        let region = ComplexRegion::new(Complex::new(-1.0, 1.0), Complex::new(1.0, -1.0));

        // Upper left corner
        let point = region.pixel_to_point(bounds, Pixel::new(0, 0));
        assert_eq!(point.re, -1.0);
        assert_eq!(point.im, 1.0);

        // Middle
        let point = region.pixel_to_point(bounds, Pixel::new(50, 100));
        assert_eq!(point.re, 0.0);
        assert_eq!(point.im, 0.0);

        // Lower right (almost - pixel 99,199 not 100,200)
        let point = region.pixel_to_point(bounds, Pixel::new(99, 199));
        assert_eq!(point.re, 0.98);
        assert_eq!(point.im, -0.99);
    }
}
