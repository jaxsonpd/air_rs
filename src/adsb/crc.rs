/// Handle crc calculation of packets
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

/// Get the crc 
/// 
/// `buf` - the data buffer (no crc)
/// 
/// returns the crc calculated from the data
pub fn get_adsb_crc(buf: &Vec<u8>) -> u32 {
    const GENERATOR: u32 = 0b1_1111_1111_1111_0100_0000_1001;
    const GENERATOR_LEN: usize = 24;

    // convert buf into a vector of bits
    let mut bits = Vec::with_capacity(buf.len() * 8 + GENERATOR_LEN);
    for byte in buf {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1 != 0);
        }
    }

    bits.extend(std::iter::repeat(false).take(GENERATOR_LEN));

    for i in 0..(bits.len() - GENERATOR_LEN) {
        if bits[i] {
            for j in 0..=GENERATOR_LEN {
                bits[i + j] ^= ((GENERATOR >> (GENERATOR_LEN - j)) & 1) != 0;
            }
        }
    }

    let mut remainder = 0u32;
    for i in 0..GENERATOR_LEN {
        if bits[bits.len() - GENERATOR_LEN + i] {
            remainder |= 1 << (GENERATOR_LEN - 1 - i);
        }
    }

    remainder
}


/// Attempt to augment the packet to match the crc
/// 
/// `buf` - the buffer to manipulate
/// `calc_crc` - the calculated crc
/// `packet_crc` - the crc from the packet
/// return a augmented buffer if succesful
pub fn try_crc_recovery(buf: Vec<u8>, calc_crc: u32, packet_crc: u32) -> Option<Vec<u8>> {
    for (num, byte) in buf.iter().enumerate() {
        let mut augmented_buf = buf.clone();
        for i in 0..8 {
            let mut augmented_byte = *byte;
            augmented_byte ^= 1 << (7-i); // flip the i-th bit
            augmented_buf[num] = augmented_byte;
            let crc = get_adsb_crc(&augmented_buf[0..augmented_buf.len()-3].to_vec());

            if crc == packet_crc {
                return Some(augmented_buf);
            }
        }
    }

    None
}