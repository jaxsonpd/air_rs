use num_complex::Complex;




/// Calculate the magnitude of every complex pair in the buffer
pub fn get_magnitude(buf: &[Complex<i16>]) -> Vec<u32> {
    let result: Vec<u32> = buf.iter()
        .map(|c| ((c.re as f64).powi(2) + (c.im as f64).powi(2)).sqrt() as u32)
        .collect();

    result
}

