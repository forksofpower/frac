use num::Complex;
use ocl::ProQue;

// Function to map the number of iterations i to a grey value between 0 (black)
// and 255 (white).
fn get_color(i: u32, max_iterations: u32) -> image::Rgb<u8> {
    if i > max_iterations {
        return image::Rgb([255, 255, 255]);
    }
    if max_iterations == 255 {
        let idx = i as u8;
        return image::Rgb([idx, idx, idx]);
    }
    let idx = (((i as f32) / (max_iterations as f32)) * 255.0).round() as u8;
    return image::Rgb([idx, idx, idx]);
}

/// Convert the given pixel to a point on the complex plane. The resulting point is the intersection of the line through
/// the upper left and lower right corners of the complex plane, and the line through the corresponding pixel and the bottom
/// edge of the image.
///
/// # Arguments
///
/// * `bounds`: A tuple representing the bounds of the image in pixels (width x height).
/// * `pixel`: A tuple representing the pixel whose corresponding point on the complex plane is to be computed (x, y).
/// * `upper_left`: A Complex number representing the upper left corner of the section of the complex plane being rendered.
/// * `lower_right`: A Complex number representing the lower right corner of the section of he complex plane being rendered.
///
/// # Returns
///
///  The point on the complex plane that corresponds to the supplied pixel.
pub fn pixel_to_point(
    bounds: (usize, usize), pixel: (usize, usize), upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);

    // map pixels's x-coordinate to real coordinate and y-coordinate to imaginary coordinate
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

/// Calculate the corners of a square in the complex plane.
///
/// Given a magnitude and center point, finds the bottom-left and top-right corners of
/// containing square in the complex plain. The corners are returned as two Complex numbers
///
/// # Arguments
///
/// * `magnitude` - A float value that determines the width and height of the square
/// * `center` - A (f64, f64) tuple representing the `(x, y)` coordinates of the center point
///
/// # Returns
///
/// Returns a tuple `(bottom_left, top_right)` representing the bottom-left and top-right corners
/// respectively. Both corners are represented as a `Complex<f64>` struct, which has fields `re`
/// (real component) and `im` (imaginary component).
///
/// # Example
/// ```
/// use num::complex::Complex;
/// use my_math_lib::calculate_corners;
/// let mag = 4.0_f64;
/// let center = (-2.0_f64, 3.0_f64);
/// let (bottom_left, top_right) = calculate_corners(mag, center);
/// assert_eq!(bottom_left, Complex::new(-4.0, 1.0));
/// assert_eq!(top_right, Complex::new(0.0, 5.0));
/// ```
pub fn calculate_corners(magnitude: f64, center: (f64, f64)) -> (Complex<f64>, Complex<f64>) {
    let half_mag = magnitude / 2.0;
    let (center_x, center_y) = center;
    (
        Complex::new(center_x - half_mag, center_y + half_mag),
        Complex::new(center_x + half_mag, center_y - half_mag),
    )
}

/// Escape time algorithm implemented to find whether the complex polynomial f(z) = z^2 + c, remain bounded or not
/// using iteration up to 'limit' times.
///
/// # Arguments
///
/// * `c` - A Complex<f64> which represents input value of the polynomial
/// * `limit` - An upper limit to the number of iterations to be performed by the algorithm
///
/// # Returns
///
/// * Some(usize) - Returns number of iterations taken for f(z) to go beyond the threshold limit (4.0).
/// * None - If limit itteration conducted and z still remain valid as per algorithm, returns none.
///
/// # Example
///
/// ```
/// use num::Complex;
/// use fractal_renderer::escape_time;
///
/// let c = Complex::new(-0.4, 0.6); // define imaginary input values
/// assert_eq!(escape_time(c, 200), Some(22)); // checks if the return value matches expected value after conducting 200 iterations
/// ```
pub fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    // Set z = 0 (initial value of z)
    let mut z = Complex { re: 0.0, im: 0.0 };
    // Iterate up to the `limit` times
    for i in 0..limit {
        if z.norm_sqr() > 32.0 {
            // Return the number of iterations it took to pass the check.
            return Some(i);
        }

        // update `z`
        z = z * z + c;
    }
    // If we have checked `limit` times without success, and z is still valid, return None
    None
}

/// Map value in range to cooresponding value in another range
fn map_ranges(value: usize, from: (usize, usize), to: (usize, usize)) -> usize {
    let range = from.1 - from.0;
    let new_range = to.1 - to.0;
    to.0 + (value - from.0) * new_range / range
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels
pub fn render(
    pixels: &mut [u8], bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>,
    limit: usize, invert: bool,
) {
    assert_eq!(pixels.len(), bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, limit) {
                None => 0,
                Some(count) => map_ranges(
                    if invert { limit - count } else { count },
                    (0, limit),
                    (0, u8::MAX.into()),
                ) as u8,
            };
        }
    }
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels using opencl
pub fn gpu_render(w: u32, h: u32, limit: usize) -> image::RgbImage {
    let mut img = image::RgbImage::new(w, h);
    // Build an OpenCL context, make it run the OpenCL C code defined in mandelbrot.cl, and
    // set the data structure to operate on as a 2D w by h structure.
    let pro_que =
        ProQue::builder().src(include_str!("mandelbrot.cl")).dims((w, h)).build().unwrap();
    // Create a buffer to be the output buffer accessible by workers.
    // This memory lives on the GPU.
    let buffer = pro_que.create_buffer::<u32>().unwrap();
    // Build the OpenCL program, make it run the kernel called `mandelbrot` and bind
    // values to the kernel arguments.
    let kernel = pro_que
        .kernel_builder("mandelbrot")
        .arg(&buffer)
        .arg(w)
        .arg(h)
        .arg(limit as u32)
        .build()
        .unwrap();

    // Run the OpenCL kernel
    unsafe {
        kernel.enq().unwrap();
    }
    let mut vec = vec![0u32; buffer.len()];
    // Copy the OpenCL buffer back to a traditional vector
    buffer.read(&mut vec).enq().unwrap();
    for (idx, iteration) in vec.iter().enumerate() {
        let rgb = get_color(*iteration, limit as u32);
        let x = idx as u32 % w;
        let y = idx as u32 / w;
        img.put_pixel(x, y, rgb);
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_to_point_upper_left() {
        let bounds = (100, 200);
        let pixel = (0, 0);
        let upper_left = Complex::new(-1.0, 1.0);
        let lower_right = Complex::new(1.0, -1.0);

        let result = pixel_to_point(bounds, pixel, upper_left, lower_right);

        assert_eq!(result.im, 1.0);
        assert_eq!(result.re, -1.0);
    }

    #[test]
    fn test_pixel_to_point_lower_right() {
        let bounds = (100, 200);
        let pixel = (99, 199);
        let upper_left = Complex::new(-1.0, 1.0);
        let lower_right = Complex::new(1.0, -1.0);

        let result = pixel_to_point(bounds, pixel, upper_left, lower_right);

        assert_eq!(result.im, -0.99);
        assert_eq!(result.re, 0.98);
    }

    #[test]
    fn test_pixel_to_point_middle() {
        let bounds = (100, 200);
        let pixel = (50, 100);
        let upper_left = Complex::new(-1.0, 1.0);
        let lower_right = Complex::new(1.0, -1.0);

        let result = pixel_to_point(bounds, pixel, upper_left, lower_right);

        assert_eq!(result.im, 0.0);
        assert_eq!(result.re, 0.0);
    }

    #[test]
    fn test_calculate_corners() {
        //within the reasonable range
        assert_eq!(
            calculate_corners(3.0, (0.0, 0.0)),
            (Complex::new(-1.5, 1.5), Complex::new(1.5, -1.5))
        );
        assert_eq!(
            calculate_corners(2.0, (-0.5, 0.0)),
            (Complex::new(-1.5, 1.0), Complex::new(0.5, -1.0))
        );
        //when magnitude is large
        assert_eq!(
            calculate_corners(1000.0, (1.0, -1.0)),
            (Complex::new(-499.0, 499.0), Complex::new(501.0, -501.0))
        );
        //when center is far away from the origin
        assert_eq!(
            calculate_corners(4.0, (5.0, -20.0)),
            (Complex::new(3.0, -18.0), Complex::new(7.0, -22.0))
        );
    }

    #[test]
    fn test_escape_time_zero() {
        let c = Complex::new(0.0, 0.0);
        assert_eq!(escape_time(c, 10), None);
    }

    #[test]
    fn test_escape_time_constant_c() {
        let c = Complex::new(1.0, 2.0);
        assert_eq!(escape_time(c, 100), Some(1));
    }

    #[test]
    fn test_escape_time_periodic_c() {
        let c = Complex::new(-0.4, 0.6);
        assert_eq!(escape_time(c, 1000), Some(26));
    }

    #[test]
    fn test_escape_time_outside_main_cardioid() {
        let c = Complex::new(-1.75, -0.02);
        assert_eq!(escape_time(c, 1000), Some(13));
    }

    #[test]
    fn test_escape_time_outside_period_2_bulb() {
        let c = Complex::new(0.32, -0.04);
        assert_eq!(escape_time(c, 1000), None);
    }

    #[test]
    fn test_map_ranges_within_range() {
        assert_eq!(map_ranges(5, (0, 10), (0, 100)), 50);
        assert_eq!(map_ranges(5, (0, 10), (50, 150)), 100);
        assert_eq!(map_ranges(10, (0, 10), (0, 50)), 50);
        assert_eq!(map_ranges(3, (0, 5), (0, 100)), 60);
    }

    // #[test]
    // fn test_render() {
    //     // Create a buffer with 16 pixels
    //     let mut pixels = vec![0u8; 16];

    //     // Define bounds and corners
    //     let width = 4;
    //     let height = 4;
    //     let upper_left = Complex::new(-1.0, 1.0);
    //     let lower_right = Complex::new(1.0, -1.0);

    //     // Render the image using 4 iterations and no inversion
    //     render(
    //         &mut pixels[..],
    //         (width, height),
    //         upper_left,
    //         lower_right,
    //         4,
    //         false,
    //     );

    //     // The expected buffer should have bright colors in the top left and bottom right corners,
    //     // and dark colors in the top right and bottom left corners.
    //     let expected_pixels = vec![255, 170, 0, 0, 170, 85, 0, 0, 0, 0, 170, 255, 0, 0, 85, 170];

    //     assert_eq!(pixels, expected_pixels);

    //     // Render the image again, but invert the color palette this time
    //     render(
    //         &mut pixels[..],
    //         (width, height),
    //         upper_left,
    //         lower_right,
    //         4,
    //         true,
    //     );

    //     // When inverted, the expected results should be flipped upside down
    //     let expected_pixels = vec![
    //         0, 85, 170, 255, 0, 170, 255, 255, 85, 255, 170, 0, 170, 255, 85, 0,
    //     ];

    //     assert_eq!(pixels, expected_pixels);
    // }
}
