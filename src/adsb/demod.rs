/// Handle demodulation of adsb messages
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use chrono::Local;
use plotters::prelude::*;

use crate::adsb::crc::{get_adsb_crc, try_crc_recovery};

const HIGH_THRESHOLD_DERATE: f64 = 0.9;

/// Check that a packet is a vaild adsb frame and is worth decoding
/// 
/// buf - the buffer to check size: 16+16=32 (preamble and first byte)
/// 
/// returns high value, signal power, noise power
pub fn check_for_adsb_packet(buf: [u32; 32]) -> Option<(u32, i32, i32)> {
    // Adsb pre amble has the following form so check it:
    //
    // +   -   +   -   -   -   -   +   -   +   -   -   -   -   -   -
    // 0  0.5  1  1.5  2  2.5  3  3.5  4  4.5  5  5.5  6  6.5  7  7.5
    // 0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15 
    let lows = [1, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15];
    let highs = [0, 2, 7, 9];
    let mut min = u32::MAX;

    for high in highs.iter() {
        for low in lows.iter() {
            if buf[*high as usize] < buf[*low as usize] {
                return None;
            }
        }
        if buf[*high as usize] < min {
            min = buf[*high as usize];
        }
    }

    // The preamble is followed by DF which needs to be 17 for adsb
    //
    // This translates too:
    //   1       0       0       0       1
    // +   -   -   +   -   +   -   +   +   -
    // 0  0.5  1  1.5  2  2.5  3  3.5  4  4.5
    // 0   1   2   3   4   5   6   7   8   9 
    let lows = [1, 2, 4, 6, 9];
    let highs = [0, 3, 5, 7, 8];

    for high in highs.iter() {
        for low in lows.iter() {
            if buf[*high + 16 as usize] < buf[*low + 16 as usize] {
                return None;
            }
        }
    }

    Some(((min as f32 * 0.9) as u32, 0, 0))
}

/// Extract a packet from a buffer of magnitude values
/// 
/// `buf` - the buffer to extract
/// `high` - the high level to use
/// 
/// returns byte vector if packet is correct and worth looking at
pub fn extract_packet(buf: Vec<u32>, high: u32) -> Option<Vec<u8>> {
    let extracted_manchester = extract_manchester_relative(buf.to_vec(), (high as f64 * HIGH_THRESHOLD_DERATE) as u32)?;

    let packet = decode_packet(extracted_manchester.clone())?;

    let len = packet.len();
    let calced_crc = get_adsb_crc(&packet[0..len-3].to_vec());
    let packet_crc = ((packet[len-1] as u32) << 0) | 
                            ((packet[len-2] as u32) << 8) |
                            ((packet[len-3] as u32) << 16);
    
    if calced_crc != packet_crc {
        // println!("Trying to recover packet with crc: {:06X}, calculated: {:06X}", packet_crc, calced_crc);
        return try_crc_recovery(packet, calced_crc, packet_crc);
    }

    Some(packet)
}

/// Extract the manchester values of a packet
/// Processed: 149, Good: 35
/// 
/// `buf` - the data buffer
/// 
/// `high` - the high level threshold (unused)
/// 
/// returns a buffer of the manchester encoding (each u16 = 8 bits in 16 edges)
fn extract_manchester_relative(buf: Vec<u32>, high: u32) -> Option<Vec<u16>> {
    let mut result = Vec::with_capacity(14);
    let mut errors = 0;

    let buf_len = buf.len();
    for block_start in (0..buf_len).step_by(16) {
        let mut symbol: u16 = 0;

        for bit in 0..8 {
            let i = block_start + bit * 2;

            let first;
            let second;

            if buf[i] > buf[i+1] {
                first = 1;
                second = 0;
            } else {
                first = 0;
                second = 1;
            }

            if first != second {
                symbol |= (first as u16) << (14 - bit*2);
                symbol |= (second as u16) << (15 - bit*2);
            } else {
                errors += 1;
                if errors > 2 {
                    return None;
                }
                // return None;
            }
        }

        result.push(symbol);
        errors = 0;
    }

    Some(result)
}

/// Extract the manchester values of a packet
/// Processed: 149, Good: 8
/// 
/// `buf` - the data buffer
/// 
/// `high` - the high level threshold
/// 
/// returns a buffer of the manchester encoding (each u16 = 8 bits in 16 edges)
fn extract_manchester_threshold(buf: Vec<u32>, high: u32) -> Option<Vec<u16>> {
    let mut result = Vec::with_capacity(14);
    let mut errors = 0;

    let buf_len = buf.len();
    for block_start in (0..buf_len).step_by(16) {
        let mut symbol: u16 = 0;

        for bit in 0..8 {
            let i = block_start + bit * 2;

            let first = buf[i] > high;
            let second = buf[i + 1] > high;

            if first != second {
                symbol |= (first as u16) << (14 - bit*2);
                symbol |= (second as u16) << (15 - bit*2);
            } else {
                errors += 1;
                if errors > 2 {
                    return None;
                }
                // return None;
            }
        }

        result.push(symbol);
        errors = 0;
    }

    Some(result)
}

/// Decode the modified Manchester encoding and return the raw hex values
/// 
/// `raw_buf` - Manchester-encoded data (each u16 = 8 bits in 16 edges)
/// 
/// returns the decoded bytes if all symbols are valid
fn decode_packet(raw_buf: Vec<u16>) -> Option<Vec<u8>> {
    let mut result = Vec::with_capacity(raw_buf.len());

    for encoded in raw_buf {
        let mut byte = 0u8;

        for i in 0..8 {
            let hi = (encoded >> (15 - i * 2)) & 1;
            let lo = (encoded >> (14 - i * 2)) & 1;

            match (hi, lo) {
                (0, 1) => byte |= 1 << (7 - i),
                (1, 0) => (),
                _ => (), // invalid Manchester bit pair
            }
        }

        result.push(byte);
    }

    Some(result)
}

/// Plot an ADS-B packet
/// 
/// - `mag_buf`: the raw magnitude vector
/// - `manchester`: the modified manchester vector
/// - `decoded`: the decoded raw hex values
pub fn plot_adsb_packet(mag_buf: Vec<u32>, manchester: Vec<u16>, decoded: Vec<u8>) {
    // Generate timestamp-based filename
    let timestamp = Local::now().format("adsb_packet_%Y%m%d_%H%M%S.svg").to_string();
    let root = SVGBackend::new(&timestamp, (1000, 400)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let max_mag = *mag_buf.iter().max().unwrap_or(&1) as f64;

    // Create chart
    let mut chart = ChartBuilder::on(&root)
        .caption("ADSB Packet", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(40)
        .build_cartesian_2d(0..mag_buf.len(), 0f64..(max_mag * 1.1))
        .unwrap();

    chart.configure_mesh().disable_mesh().draw().unwrap();

    // Plot raw magnitude as bars
    chart.draw_series(
        mag_buf
            .iter()
            .enumerate()
            .map(|(x, y)| {
                let y = *y as f64;
                Rectangle::new(
                    [(x, 0.0), (x + 1, y)],
                    BLUE.mix(0.5).filled(),
                )
            })
    ).unwrap();

    println!("Plot saved to '{}'", timestamp);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_for_adsb_packet_valid() {
        // Construct a buffer with clear high/low structure per ADS-B preamble
        let mut buf = [0u32; 32];
        for &i in &[0, 2, 7, 9] {
            buf[i] = 1000; // highs
        }
        for &i in &[1, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15] {
            buf[i] = 500; // lows
        }

        let result = check_for_adsb_packet(buf);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, (1000.0 * 0.9) as u32);
    }

    #[test]
    fn test_check_for_adsb_packet_invalid() {
        // One low is higher than high — should return None
        let mut buf = [0u32; 32];
        for &i in &[0, 2, 7, 9] {
            buf[i] = 500; // highs
        }
        for &i in &[1, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15] {
            buf[i] = 1000; // lows
        }

        assert_eq!(check_for_adsb_packet(buf), None);
    }

    #[test]
    fn test_extract_manchester_valid() {
        // Simulate a repeating pattern of 1s and 0s
        let mut buf = vec![0; 224];
        let high = 100;

        for i in (0..224).step_by(4) {
            buf[i] = 120;     // high
            buf[i + 1] = 50;  // low
            buf[i + 2] = 50;  // low
            buf[i + 3] = 120; // high
        }

        let result = extract_manchester_threshold(buf, high);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 14);
    }

    #[test]
    fn test_extract_manchester_invalid() {
        // Simulate a repeating pattern of 1s and 0s
        let mut buf = vec![0; 224];
        let high = 100;

        for i in (0..224).step_by(4) {
            buf[i] = 120;     // high
            buf[i + 1] = 50;  // low
            buf[i + 2] = 50;  // low
            buf[i + 3] = 120; // high
        }

        buf[0] = 50;
        buf[1] = 50;
        buf[2] = 120;
        buf[3] = 120; 
        buf[4] = 50;
        buf[5] = 50;

        let result = extract_manchester_threshold(buf, high);
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_packet_valid() {
        // 0b10 = 1, 0b01 = 0 -> produces 0b10101010 = 0xAA
        let encoded = vec![0b1001100110011001];
        let result = decode_packet(encoded);
        assert_eq!(result, Some(vec![0xAA]));
    }

    #[test]
    fn test_decode_packet_invalid() {
        // 0b00 or 0b11 are invalid Manchester codes
        let encoded = vec![0b1111000011110000];
        assert_eq!(decode_packet(encoded), None);
    }

    #[test]
    fn test_get_adsb_crc() {
        let data = vec![
            0x8D, 0x40, 0x6B, 0x90, 0x20, 0x15,
            0xA6, 0x78, 0xD4, 0xD2, 0x20
        ];
        let crc = get_adsb_crc(&data);
        assert_eq!(crc, 0xAA4BDA);
    }

    #[test]
    fn test_get_adsb_crc_real() {
        let buf = vec![0x8d, 0x40, 0x6b, 0x90, 0x20, 0x15, 0xa6, 0x78, 
                        0xd4, 0xd2, 0x20]; 
        
        let crc = get_adsb_crc(&buf);

        assert_eq!(crc, 0xaa4bda);

    }

    #[test]
    fn test_get_adsb_crc_real_invalid() {
        let buf = vec![0x8d, 0x40, 0x6a, 0x90, 0x20, 0x15, 0xa6, 0x78, 
                        0xd4, 0xd2, 0x20]; 
        
        let crc = get_adsb_crc(&buf);

        assert_ne!(crc, 0xaa4bda);

    }

    #[test]
    fn test_extract_packet_bad_crc() {
        let mut buf = Vec::new();
        let high = 100;
        for i in (0..224).step_by(2) {
            buf.push(120);
            buf.push(50);
        }

        // This will decode to a valid pattern but incorrect CRC
        assert!(extract_packet(buf, high).is_none());
    }
}