__kernel void mandelbrot(__global uint* buffer, uint width, uint height, uint max_iterations) {
    // get the x coordinate of this worker. We can get the x and y coodinates because the kernel 
    // operates over a 2-dimensional data struction.
    int c = get_global_id(0);
    // get the y coordinate of this worker.
    int r = get_global_id(1);
    // naive implementation optimized to only use 3 multiplications in the inner loop.
    float x0 = ((float)c / width) * 3.5 - 2.5;
    float y0 = ((float)r / height) * 2.0 - 1.0;
    float x = 0.0;
    float y = 0.0;
    float x2 = 0.0;
    float y2 = 0.0;
    uint iteration = 0;
    while (((x2 + y2) <= 4.0) && (iteration < max_iterations)) {
        y = (x + x) * y + y0;
        x = x2 - y2 + x0;
        x2 = x * x;
        y2 = y * y;
        iteration = iteration + 1;
    }
    // Store the number of iterations computed by this worker.
    buffer[width * r + c] = iteration;
}