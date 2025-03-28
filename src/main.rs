mod mandelbrot;
mod parsers;
mod algorithms;
use mandelbrot::Canvas;
mod cli;
use cli::Arguments;

// #[macro_use]
extern crate clap;
use clap::Parser;

#[cfg(feature = "gpu")]
extern crate ocl;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "gpu")]
use gpu::gpu_render;

use image::png::PNGEncoder;
use image::ColorType;

use std::fs::File;
use rayon::prelude::*;

/// Write the buffer `pixels`, whose dimensions are given by `bounds` to the file name `filename`
fn write_image(
    filename: &str, pixels: &[u8], bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())
}



fn main() {
    let args = Arguments::parse();
    let (upper_left, lower_right) = mandelbrot::calculate_corners(args.zoom, args.center);

    if cfg!(feature = "gpu") && args.gpu {
        #[cfg(feature = "gpu")]
        {
            let img = gpu_render(args.dimensions.0 as u32, args.dimensions.1 as u32, args.limit);
            let file_prefix = "gpu_";
            let filename = format!("{}{}", file_prefix, args.output);
            // write_image(&args.output, &output, arg.dimensions).expect("error writing gpu PNG file");
            img.save(filename).expect("error writing GPU PNG file");
            // println!("{:?}", img);
        }
    } else {
        let mut pixels = vec![0; args.dimensions.0 * args.dimensions.1];
        let plotter = crate::algorithms::get_plotting_algorithm(&args.algorithm);
        let canvas = Canvas::new(plotter);
        
        let bands: Vec<(usize, &mut [u8])> = pixels
            .chunks_mut(args.dimensions.0)
            .enumerate()
            .collect();

        bands.into_par_iter().for_each(|(i, band)| {
            let top = i;
            let band_bounds = (args.dimensions.0, 1);
            let band_upper_left = mandelbrot::pixel_to_point(
                args.dimensions,
                (0, top),
                upper_left,
                lower_right,
            );
            let band_lower_right = mandelbrot::pixel_to_point(
                args.dimensions,
                (args.dimensions.0, top + 1),
                upper_left,
                lower_right,
            );

            canvas.render(
                band,
                band_bounds,
                band_upper_left,
                band_lower_right,
                args.limit,
                args.invert,
            );
        });

        write_image(&args.output, &pixels, args.dimensions).expect("error writing PNG file");
    }
}
