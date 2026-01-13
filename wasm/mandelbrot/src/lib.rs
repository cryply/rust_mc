


use wasm_bindgen::prelude::*;
use rayon::prelude::*;
use num_complex::Complex;

pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn mandelbrot(width: u32, height: u32, max_iter: u32) -> Vec<u8> {
    
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    // Size of one row in bytes
    let bytes_per_row = (width * 4) as usize;


    pixels
        .par_chunks_mut(bytes_per_row)
        .enumerate()
        .for_each(|(row_idx, chunk)| {
            for xi in 0..width {
                let idx = (xi as usize * 4) as usize;
                let c = Complex::new(
                    (xi as f64 / width as f64 * 3.5) - 2.5,
                    (row_idx as f64 / height as f64 * 2.0) - 1.0,
                );
                let i = iter(&c, max_iter);
                let color = ((i as f64 / max_iter as f64) * 255.0) as u8;
                chunk[idx] = 255;      // R
                chunk[idx + 1] = color; // G
                chunk[idx + 2] = 0;     // B
                chunk[idx + 3] = 255;   // A
            }
    });
    pixels
}

fn iter(c: &Complex<f64>, max: u32) -> u32 {
    let mut z = Complex::new(0.0, 0.0);
    let mut i = 0u32;
    while z.norm_sqr() <= 4.0 && i < max {
        z = z * z + *c;
        i += 1;
    }
    i
}
