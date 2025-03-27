use num::{Complex, complex::ComplexFloat};

pub trait PlottingAlgorithm {
    fn calculate(&self, c: Complex<f64>, zoom: usize, limit: usize) -> Option<usize>;
}

pub struct EscapeTime;
impl PlottingAlgorithm for EscapeTime {
    fn calculate(&self, c: Complex<f64>, _zoom: usize, limit: usize) -> Option<usize> {
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
}

pub struct BurningShip;
impl PlottingAlgorithm for BurningShip {
    fn calculate(&self, c: Complex<f64>, _zoom: usize, limit: usize) -> Option<usize> {
        let mut z = Complex::new(0.0, 0.0);
        let mut iterations = 0;

        while z.norm_sqr() <= 4.0 && iterations < limit {
            // Note the .abs() calls on the components
            z = Complex::new(z.im.abs(), z.im.abs()); 
            z = z * z + c;
            iterations += 1;
        }
        if iterations == limit {
            None
        } else {
            Some(iterations)
        }
    }
}

pub fn get_plotting_algorithm(name: &str) -> Box<dyn PlottingAlgorithm + Send + Sync> {
    match name {
        "escape_time" => Box::new(EscapeTime),
        "burning_ship" => Box::new(BurningShip),
        _ => Box::new(EscapeTime), // default to EscapeTime if unknown
    }
}