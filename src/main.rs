mod mandelbrot;
mod parsers;
use parsers::parse_pair;

#[macro_use]
extern crate clap;

#[cfg(feature = "gpu")]
extern crate ocl;

#[cfg(feature = "gpu")]
mod gpu;

#[cfg(feature = "gpu")]
use gpu::gpu_render;

use clap::Parser;
use image::png::PNGEncoder;
use image::ColorType;

use std::fs::File;
use std::sync::Mutex;

/// Write the buffer `pixels`, whose dimensions are given by `bounds` to the file name `filename`
fn write_image(
    filename: &str, pixels: &[u8], bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    zoom: f64,

    #[arg(short, long, default_value = "mandelbrot.png")]
    output: String,

    #[arg(
        short,
        long,
        allow_hyphen_values = true,
        value_parser = |arg: &str| match parse_pair::<f64>(arg, ',') {
            Some(v) => Ok(v),
            None => Err("error parsing center point".to_string())
        }
    )]
    center: (f64, f64),

    #[arg(
        short,
        long,
        default_value = "1920x1080",
        value_parser = |arg: &str| match parse_pair::<usize>(arg, 'x') {
            Some(v) => Ok(v),
            None => Err("error parsing image dimensions".to_string())
        }
    )]
    dimensions: (usize, usize),

    #[arg(short, long)]
    gpu: bool,

    #[arg(short, long)]
    limit: usize,

    #[arg(short, long)]
    invert: bool,
}
// fn build_command() -> Command {
//     let zoom = Arg::new("zoom").short('z').help("zoom level");
//     let gpu = Arg::new("gpu").num_args(0).help("enable gpu rendering");
//     let invert = Arg::new("invert").short('i').num_args(0).help("invert color rendering");
//     let center =
//         Arg::new("center").short('c').help("center coordinates").value_parser(|arg: &str| {
//             match parse_pair::<f64>(arg, ',') {
//                 Some(v) => Ok(v),
//                 None => Err("error parsing center point".to_string()),
//             }
//         });
//     let dimensions =
//         Arg::new("dimensions").short('d').help("center coordinates").value_parser(|arg: &str| {
//             match parse_pair::<usize>(arg, 'x') {
//                 Some(v) => Ok(v),
//                 None => Err("error parsing image dimensions".to_string()),
//             }
//         });

//     let command = Command::new(crate_name!())
//         .version(crate_version!())
//         .author(crate_authors!())
//         .arg(zoom)
//         .arg(center)
//         .arg(dimensions)
//         .arg(invert)
//         .limit;

//     #[cfg(feature = "gpu")]
//     command.arg(gpu);

//     return command;
// }
fn main() {
    let args = Arguments::parse();
    let (upper_left, lower_right) = mandelbrot::calculate_corners(args.zoom, args.center);

    if cfg!(feature = "gpu") && args.gpu {
        #[cfg(feature = "gpu")]
        {
            let output = gpu_render(8, 8, 256);
            println!("{:?}", output);
        }
    } else {
        let mut pixels = vec![0; args.dimensions.0 * args.dimensions.1];
        let threads = 8;
        let rows_per_band = args.dimensions.1 / threads + 1;

        let bands = Mutex::new(pixels.chunks_mut(rows_per_band * args.dimensions.0).enumerate());

        // Spawn workers and have them pull bands to work on until finished.
        crossbeam::scope(|spawner| {
            for _ in 0..threads {
                spawner.spawn(|_| loop {
                    match {
                        let mut guard = bands.lock().unwrap();
                        guard.next()
                    } {
                        None => {
                            return;
                        }
                        Some((i, band)) => {
                            let top = rows_per_band * i;
                            let height = band.len() / args.dimensions.0;
                            let band_bounds = (args.dimensions.0, height);
                            let band_upper_left = mandelbrot::pixel_to_point(
                                args.dimensions,
                                (0, top),
                                upper_left,
                                lower_right,
                            );
                            let band_lower_right = mandelbrot::pixel_to_point(
                                args.dimensions,
                                (args.dimensions.0, top + height),
                                upper_left,
                                lower_right,
                            );

                            mandelbrot::render(
                                band,
                                band_bounds,
                                band_upper_left,
                                band_lower_right,
                                args.limit,
                                args.invert,
                            );
                        }
                    }
                });
            }
        })
        .unwrap();

        write_image(&args.output, &pixels, args.dimensions).expect("error writing PNG file");
    }
}
