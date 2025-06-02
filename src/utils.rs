use num_complex::Complex;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn save_data(data: &[Complex<i16>]) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let name = format!("test_{now}.iq");
    let mut file = std::fs::File::create(name).unwrap();

    for d in data {
        file.write_i16::<LittleEndian>(d.im).unwrap();
        file.write_i16::<LittleEndian>(d.re).unwrap();
    }
}

/// Calculate the magnitude of every complex pair in the buffer
pub fn get_magnitude(buf: &[Complex<i16>]) -> Vec<u32> {
    let result: Vec<u32> = buf.iter()
        .map(|c| ((c.re as f64).powi(2) + (c.im as f64).powi(2)).sqrt() as u32)
        .collect();

    result
}

