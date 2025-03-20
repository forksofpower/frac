extern crate ocl;

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
