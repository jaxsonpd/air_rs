/// Implementation for the adsb packet structure and handling

use crate::adsb_msgs::{AdsbMsgType, AircraftID, UknownMsg, AircarftPosition, AdsbMsg};

use chrono::Local;

#[derive(Debug, Clone)]
pub struct AdsbPacket {
    raw_manchester: Vec<u16>,
    packet: Vec<u8>,
    downlink_format: u8,
    capability: u8,
    pub icao: u32,
    pub msg_type: u8,
    pub msg: AdsbMsgType,
    pub time_processed: chrono::prelude::DateTime<Local>
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

        let msg;
        if AircraftID::msg_id_match(msg_type) {
            msg = AdsbMsgType::AircraftID(AircraftID::new(packet[4..4+7].try_into().expect(format!("Bad aircraft id packet: {:?}", packet).as_str())));
        } else if AircarftPosition::msg_id_match(msg_type) {
            msg = AdsbMsgType::AircarftPosition(AircarftPosition::new(packet[4..4+7].try_into().expect(format!("Bad aircraft id packet: {:?}", packet).as_str())));
        } else {
            msg = AdsbMsgType::Uknown(UknownMsg {raw_msg: packet[4..packet.len()].to_vec()});
        }

        Self {
            raw_manchester: raw_buf,
            packet: packet.clone(),
            downlink_format: downlink_format,
            capability: capability,
            icao: icao,
            msg_type: msg_type,
            msg: msg,
            time_processed: Local::now()
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

    pub fn get_icao(&self) -> u32 {
        self.icao
    }
}

impl std::fmt::Display for AdsbPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Convert packet to hex string (2 hex chars per byte)
        let packet_hex: String = self.packet
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join("");

        writeln!(f, "{}", packet_hex)?;

        // Add the decoded metadata
        writeln!(f, "\nDecoded Information:")?;
        writeln!(f, "Downlink Format : {}", self.downlink_format)?;
        writeln!(f, "Capability      : {}", self.capability)?;
        writeln!(f, "ICAO            : {:06X}", self.icao)?;
        writeln!(f, "Processed Time  : {}", self.time_processed)?;
        writeln!(f, "Message Type    : {}", self.msg_type)?;
        write!(f, "{}", self.msg)?;

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