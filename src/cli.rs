use clap::{Parser, builder::PossibleValuesParser};

use crate::parsers::parse_pair;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long)]
    pub zoom: f64,

    #[arg(short, long, default_value = "mandelbrot.png")]
    pub output: String,

    #[arg(
        short, 
        long, 
        default_value = "escape_time",
        value_parser = PossibleValuesParser::new(["escape_time", "burning_ship"])
    )]
    pub algorithm: String,

    #[arg(
        short,
        long,
        allow_hyphen_values = true,
        value_parser = |arg: &str| match parse_pair::<f64>(arg, ',') {
            Some(v) => Ok(v),
            None => Err("error parsing center point".to_string())
        }
    )]
    pub center: (f64, f64),

    #[arg(
        short,
        long,
        default_value = "1920x1080",
        value_parser = |arg: &str| match parse_pair::<usize>(arg, 'x') {
            Some(v) => Ok(v),
            None => Err("error parsing image dimensions".to_string())
        }
    )]
    pub dimensions: (usize, usize),

    #[arg(short, long)]
    pub gpu: bool,

    #[arg(short, long)]
    pub limit: usize,

    #[arg(short, long)]
    pub invert: bool,
}