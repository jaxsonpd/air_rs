use num_complex::Complex;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

/// Save complex data in SatDump-compatible `.c16` format (I then Q)
pub fn save_data(data: &[Complex<i16>], name: String) -> Result<(), Box<dyn std::error::Error>> {

    let mut writer = BufWriter::new(File::create(name)?);

    let mut buf = Vec::with_capacity(data.len() * 2 * std::mem::size_of::<i16>());
    for c in data {
        buf.write_i16::<LittleEndian>(c.re)?;
        buf.write_i16::<LittleEndian>(c.im)?;
    }

    writer.write_all(&buf)?;
    writer.flush()?;
    Ok(())
}

/// Load complex data from `.c16` format (I then Q)
pub fn load_data(filename: String) -> Result<Vec<Complex<i16>>, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(filename)?);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    if bytes.len() % 4 != 0 {
        return Err("Invalid file length (not divisible by 4)".into());
    }

    let mut data = Vec::with_capacity(bytes.len() / 4);
    let mut cursor = std::io::Cursor::new(bytes);

    while let (Ok(re), Ok(im)) = (
        cursor.read_i16::<LittleEndian>(),
        cursor.read_i16::<LittleEndian>(),
    ) {
        data.push(Complex::new(re, im));
    }

    Ok(data)
}

/// Calculate the magnitude of every complex pair in the buffer
pub fn get_magnitude(buf: &[Complex<i16>]) -> Vec<u32> {
    let result: Vec<u32> = buf.iter()
        .map(|c| ((c.re as f64).powi(2) + (c.im as f64).powi(2)).sqrt() as u32)
        .collect();

    result
}

