use image::png::PNGEncoder;
use image::ColorType;
use num::Complex;
use std::{env, fs::File, str::FromStr};

// Parsers for command line args
/// A generic function to parse a pair from a string separated by a given character.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    // Find the index of the separator in the string.
    match s.find(separator) {
        None => None, // If no separator is found return None.
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            // If both left and right parts of the string can be parsed into T then return them as a tuple wrapped in a Some.
            (Ok(left), Ok(right)) => Some((left, right)),
            _ => None, // else return None if there is a parsing error on either side.
        },
    }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
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
fn pixel_to_point(
    bounds: (usize, usize),   // The bounds of the image in pixels (width x height)
    pixel: (usize, usize),    // The pixel to map (x, y)
    upper_left: Complex<f64>, // The upper left corner of the section of the complex plane being rendered.
    lower_right: Complex<f64>, // The lower right corner of the section of he complex plane being rendered.
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );

    // map pixels's x-coordinate to real coordinate and y-coordinate to imaginary coordinate
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

/// A function that determines whether a complex number `c` belongs to the Mandelbrot set.
/// If the number of iterations reaches the limit before |z|^2 > 4, returns None. Otherwise,
/// returns the number of iterations needed to reach |z|^2 > 4.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    // Set z = 0 (initial value of z)
    let mut z = Complex { re: 0.0, im: 0.0 };
    // Iterate up to the `limit` times
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            // Return the number of iterations it took to pass the check.
            return Some(i);
        }

        // update `z`
        z = z * z + c;
    }
    // If we have checked `limit` times without success, and z is still valid, return None
    None
}
/// Render a rectangle of the Mandelbrot set into a buffer of pixels
fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert_eq!(pixels.len(), bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            };
        }
    }
}

/// Write the buffer `pixels`, whose dimensions are given by `bounds` to the file name `filename`
fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())
}

fn print_help() {
    let args: Vec<String> = env::args().collect();

    eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT", args[0]);
    eprintln!(
        "Example:\n {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
        args[0]
    );
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        print_help()
    }

    let filename = &args[1];
    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();

    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = rows_per_band * i;
            let height = band.len() / bounds.0;
            let band_bounds = (bounds.0, height);
            let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);

            spawner.spawn(move |_| render(band, band_bounds, band_upper_left, band_lower_right));
        }
    })
    .unwrap();

    write_image(filename, &pixels, bounds).expect("error writing PNG file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair::<i32>("", ','), None);
        assert_eq!(parse_pair::<i32>("10,", ','), None);
        assert_eq!(parse_pair::<i32>(",5", ','), None);
        assert_eq!(parse_pair::<i32>("10,5", ','), Some((10, 5)));
        assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
        assert_eq!(parse_pair::<i32>("0.5x", 'x'), None);
        assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(
            parse_complex("1.25,-0.0625"),
            Some(Complex {
                re: 1.25,
                im: -0.0625
            })
        );
        assert_eq!(parse_complex(",-0.0625"), None)
    }

    #[test]
    fn test_pixel_to_point() {
        assert_eq!(
            pixel_to_point(
                (100, 200),
                (25, 175),
                Complex { re: -1.0, im: 1.0 },
                Complex { re: 1.0, im: -1.0 }
            ),
            Complex {
                re: -0.5,
                im: -0.75
            }
        )
    }

    #[test]
    fn test_render_empty_buffer() {
        // Test that calling render with an empty buffer does not panic.
        let mut pixels = [];
        render(
            &mut pixels,
            (0, 0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        );
    }

    #[test]
    fn test_render_full_buffer() {
        // Test that calling render with a full buffer does not panic.
        let mut pixels = vec![0; 100 * 50];
        render(
            &mut pixels,
            (100, 50),
            Complex::new(-1.5, 1.5),
            Complex::new(0.5, -1.5),
        );
    }

    // #[test]
    // fn test_render_single_pixel() {
    //     // Test that calling render with bounds of size one renders a correct pixel color.
    //     let mut pixels = [0];
    //     render(
    //         &mut pixels,
    //         (1, 1),
    //         Complex::new(-2.0, 2.0),
    //         Complex::new(2.0, -2.0),
    //     );
    //     assert_eq!(pixels[0], 255);
    // }

    // #[test]
    // fn test_render_color_bounds() {
    //     // Test that calling render with given upper-left and lower-right coordinates sets colors correctly.
    //     let mut pixels = vec![0; 6];
    //     render(
    //         &mut pixels,
    //         (3, 2),
    //         Complex::new(0.0, 10.0),
    //         Complex::new(4.0, 6.0),
    //     );
    //     assert_eq!(pixels, [0, 127, 255, 0, 63, 127]);
    // }

    #[test]
    fn test_escape_time() {
        let c = Complex { re: -0.4, im: 0.6 };
        assert_eq!(escape_time(c, 100), Some(26));
    }
}
