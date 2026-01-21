# :cyclone: Frac: High-Performance Fractal Renderer

A command-line tool written in Rust for rendering high-resolution fractal images (currently Mandelbrot and Burning Ship sets). This project leverages parallel CPU processing (via Rayon) and optional GPU acceleration (via OpenCL) to generate images quickly.

## Features
- Multiple Algorithms: Supports standard Mandelbrot (Escape Time) and Burning Ship fractals.
- High Performance:
    - CPU: Multithreaded rendering using rayon to utilize all available cores.
    - GPU: Optional OpenCL acceleration for massive speedups on compatible hardware.
- Customizable Output:
    - Set custom image dimensions.
    - Zoom into specific coordinates.
    - Adjust iteration limits for higher detail.
    - Invert colors.
    - Portable: outputs standard PNG images.

## Installation

### Prerequisites
- Rust and Cargo installed.
- (Optional) OpenCL drivers installed for your graphics card (NVIDIA CUDA toolkit, AMD ROCm, or Intel OpenCL Runtime) if you intend to use GPU rendering.

## Building

1. Standard CPU Build (Default)
   `cargo build --release`

2. GPU Enabled Build
   To enable the GPU functionality, you must build with the gpu feature flag. This requires OpenCL libraries to be present on your system.
   `cargo build --release --features gpu`

## Usage

Run the binary using `cargo run` or directly via the generated executable in `target/release/`.
`cargo run --release -- [OPTIONS]`

## Command Line Arguments

All arguments are optional and have sensible defaults.

| Flag | Long Flag | Default | Description |
|------|-----------|---------|-------------|
| `-f` | `--output` | `mandelbrot.png` | The filename to save the resulting image to. |
| `-z` | `--zoom` *(required)* | 1.0 | The zoom level/magnification factor. |
| `-c` | `--center` *(required)* | 0.0,0.0 | Center coordinates in the complex plane (e.g., -0.5,0.0). |
| `-d` | `--dimensions` | 1920x1080 | Output image resolution (Width x Height). |
| `-l` | `--limit` *(required)* | 100 | The maximum number of iterations per pixel (determines detail/brightness). |
| `-a` | `--algorithm` | escape_time | The fractal algorithm to use. Options: escape_time, burning_ship. |
| `-i` | `--invert` | false | Invert the color intensity. |
| `-g` | `--gpu` | false   | Enable GPU rendering (requires build with --features gpu). |
| `-h` | `--help`     || Print help information.

## Examples

1. Basic Mandelbrot Render
   Render a standard view of the Mandelbrot set to mandelbrot.png.
   ```
   cargo run --release -- --zoom 1.0 --center -0.5,0.0 --limit 100
   ```

2. High-Resolution GPU Render
   Render a 4K image using the GPU (requires --features gpu during build).
   ```
   cargo run --release --features gpu -- \
    --gpu \
    --dimensions 3840x2160 \
    --zoom 1.0 \
    --center -0.5,0.0 \
    --limit 500 \
    --output gpu_render.png
    ```

3. Burning Ship Fractal
Switch algorithms to render the "Burning Ship" fractal.
   ```
   cargo run --release -- \
    --algorithm burning_ship \
    --zoom 0.8 \
    --center -1.75,0.03 \
    --limit 255 \
    --output ship.png
    ```
4. Deep Zoom
Zooming into a specific interesting area.
   ```
   cargo run --release -- \
    --center -0.7436438870371587,0.1318259042053119 \
    --zoom 6000.0 \
    --limit 2000 \
    --output zoom.png
    ```
Project Structure
- __src/main.rs__: Entry point. Handles argument parsing and dispatches to CPU or GPU renderers.
- __src/cli.rs__: Defines command-line arguments using clap.
- __src/algorithms.rs__: Implementation of EscapeTime and BurningShip algorithms.
- __src/mandelbrot.rs__: Coordinate mapping logic (pixel_to_point, calculate_corners) and CPU canvas rendering.
- __src/gpu.rs__: OpenCL setup and kernel execution wrapper.
- __src/shaders/mandelbrot.cl__: The OpenCL C kernel code that runs on the GPU.
- __src/parsers.rs__: Helpers for parsing command line strings (e.g., "1920x1080").
Testing
The project includes unit tests for coordinate systems and parsers. Run them with: `cargo test`
