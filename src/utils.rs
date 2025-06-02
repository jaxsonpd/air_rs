use num_complex::Complex;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn save_data(data: &[Complex<i16>]) -> Result<(), Box<dyn std::error::Error>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let name = format!("test_{now}.iq");
    let mut file = std::fs::File::create(name)?;

    for d in data {
        file.write_i16::<LittleEndian>(d.im).unwrap();
        file.write_i16::<LittleEndian>(d.re).unwrap();
    }

    Ok(())
}

pub fn load_data(filename: String) -> Result<Vec<Complex<i16>>, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(filename)?;
    let mut r_buf: Vec<Complex<i16>> = Vec::new();

    loop {
        let im = match file.read_i16::<LittleEndian>() {
            Ok(val) => val,
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        };

        let re = match file.read_i16::<LittleEndian>() {
            Ok(val) => val,
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        };

        r_buf.push(Complex::new(re, im));
    }

    Ok(r_buf)
}

/// Calculate the magnitude of every complex pair in the buffer
pub fn get_magnitude(buf: &[Complex<i16>]) -> Vec<u32> {
    let result: Vec<u32> = buf.iter()
        .map(|c| ((c.re as f64).powi(2) + (c.im as f64).powi(2)).sqrt() as u32)
        .collect();

    result
}

