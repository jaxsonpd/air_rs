use std::{default, os::linux::raw};


#[derive(Debug)]
pub enum DownlinkFormat {
    Civil = 17,
}

#[derive(Debug)]
pub enum Capability {
    LevelZero = 0,
    LevelTwo = 4,
}

#[derive(Debug)]
pub enum MsgType {
    ID = 0,
    SurfacePosition,
    AirbornPosBaro,
    AirbornVel,
    AirbornPosGNSS,
    Reserved,
    AircraftStatus,
    TargetStatus,
    AircraftOperationStatus
}

trait AdsbMsg: std::fmt::Debug {
    fn new(msg: Vec<u8>) -> Self;
}

#[derive(Debug)] 
pub struct AircraftID {
    raw_msg: Vec<u8>,
    callsign: String
}

fn to_6bit_chunks(input: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;

    for byte in input {
        acc = (acc << 8) | byte as u32;
        bits += 8;

        while bits >= 6 {
            bits -= 6;
            out.push(((acc >> bits) & 0x3F) as u8);
        }
    }

    if bits > 0 {
        out.push(((acc << (6 - bits)) & 0x3F) as u8);
    }

    out
}

impl AdsbMsg for AircraftID {
    fn new(msg: Vec<u8>) -> Self {
        let char_convert: Vec<char> = "#ABCDEFGHIJKLMNOPQRSTUVWXYZ#####_###############0123456789######".chars().collect();
        let msg_6_bit = to_6bit_chunks(msg[1..].to_vec());
        let mut callsign: String = String::new();

        for byte in msg_6_bit.iter() {
            if let Some(&ch) = char_convert.get(*byte as usize) {
                callsign.push(ch);
            } else {
                callsign.push('?'); // fallback if index is out of bounds
            }
        }

        Self {
            raw_msg: msg,
            callsign: callsign

        }

    }
}

#[derive(Debug)]
pub struct AdsbPacket {
    raw_manchester: Vec<u16>,
    packet: Vec<u8>,
    downlink_format: u8,
    capability: u8,
    icao: u32,
    msg_type: u8,
    msg: Box<dyn AdsbMsg>
}



impl AdsbPacket {
    /// Create a new adsb packet and perform decoding
    /// 
    /// raw_buf - the raw no simplified modified manchester buffer
    /// 
    pub fn new(raw_buf: Vec<u16>) -> AdsbPacket{
        let packet = AdsbPacket::decode_packet(&raw_buf);
        
        let downlink_format = packet[0] >> 3;
        let capability = packet[0] & 5;
        let icao: u32 = (packet[1] as u32) << 16 | (packet[2] as u32) << 8 | packet[3] as u32;
        let msg_type = packet[4] >> 3;

        Self {
            raw_manchester: raw_buf,
            packet: packet.clone(),
            downlink_format: downlink_format,
            capability: capability,
            icao: icao,
            msg_type: msg_type,
            msg: packet[4..packet.len()].to_vec(),
        }
    }

    /// Decode the modifided manchester encoding and return the 
    /// raw hex values
    /// 
    /// raw_buf - the raw modified manchester buffer
    /// returns the packet in hex form
    fn decode_packet(raw_buf: &Vec<u16>) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let mut inter: u8 = 0;

        for byte in raw_buf.iter() {
            for i in (0..16).step_by(2) {
                let bits = (byte >> (14 - i)) & 0x2;

                match bits {
                    0b10 => inter |= 1 << (7 - (i/2)),
                    _ => inter &= !(1 << (7 - (i/2)))
                }
            }
            result.push(inter);
            inter = 0;
        }
        result
    }
}

use std::fmt;

use clap::builder::Str;

impl fmt::Display for AdsbPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert raw_manchester to a binary string with space between 16-bit blocks
        let raw_binary: String = self.raw_manchester
            .iter()
            .map(|word| format!("{:016b}", word))
            .collect::<Vec<_>>()
            .join(" ");

        // Convert packet to binary string (8 bits per byte)
        let packet_binary: String = self.packet
            .iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<_>>()
            .join("");

        // Convert packet to hex string (2 hex chars per byte)
        let packet_hex: String = self.packet
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join("");

        // Compute the visual offset due to raw_binary spacing
        let total_bits = self.raw_manchester.len() * 16;
        let spaces_between = self.raw_manchester.len().saturating_sub(1);
        let pad_len = total_bits + spaces_between;

        // writeln!(f, "{}", raw_binary)?;
        // writeln!(f, "{:>width$}", packet_binary, width = pad_len)?;
        // writeln!(f, "{:>width$}", packet_hex, width = pad_len / 4)?;

        writeln!(f, "{}", packet_hex)?;

        // Add the decoded metadata
        writeln!(f, "\nDecoded Information:")?;
        writeln!(f, "Downlink Format : {}", self.downlink_format)?;
        writeln!(f, "Capability      : {}", self.capability)?;
        writeln!(f, "ICAO            : {:06X}", self.icao)?;
        writeln!(f, "Message Type    : {}", self.msg_type)?;
        writeln!(f, "Message         : {:?}", self.msg)?;

        Ok(())
    }
}


pub fn check_preamble(buf: Vec<u32>) -> Option<(u32, i32, i32)> {
    assert!(buf.len() == 16);

    // Adsb pre amble has the following form:
    //
    // +   -   +   -   -   -   -   +   -   +   -   -   -   -   -   -
    // 0  0.5  1  1.5  2  2.5  3  3.5  4  4.5  5  5.5  6  6.5  7  7.5
    // 0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15 
    let lows = [1, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15];
    let highs = [0, 2, 7, 9];
    let mut min = 800000;

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


    Some(((min as f32 * 0.9) as u32, 0, 0))
}

pub fn check_df(buf: Vec<u32>) -> bool {
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
            if buf[*high as usize] < buf[*low as usize] {
                return false;
            }
        }
    }

    true
}

/// Extract the manchester values of a packet
/// 
/// buf - the data buffer
/// high - the high level threshold
/// 
/// returns the manchester bits if there are not to many errors
pub fn extract_manchester(buf: Vec<u32>, high: u32) -> Option<Vec<u16>> {
    let mut result: Vec<u16> = Vec::new();
    let mut inter: u16 = 0;
    let mut errors: u8 = 0;

    for i in (0..112*2).step_by(2) {
        if errors > 2 {
            return None;
        }

        if i % 16 == 0 && i != 0{
            result.push(inter);
            inter = 0;
            errors = 0;
            print!(" ");
        }

        inter |= ((buf[i] > high) as u16) << (15 - i % 16);


        if buf[i] > high && buf[i+1] < high { // 1
            continue;
        } else if buf[i] < high && buf[i+1] > high {
            continue;
        } else {
            errors += 1;
        }
    }

    Some(result)
}