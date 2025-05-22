/// Definitions for the adsb message types
///
/// Jack Duignan (JackpDuignan@gmail.com)

/// Top level enum to hold various message types
#[derive(Debug)]
pub enum AdsbMsgType {
    AircraftID(AircraftID),
    AircarftPosition(AircarftPosition),
    Uknown(UknownMsg)
}

/// Top level trait for messages
pub trait AdsbMsg: std::fmt::Debug {
    fn msg_id_match(id: u8) -> bool where Self: Sized;
}

#[derive(Debug)]
pub struct UknownMsg {
    pub raw_msg: Vec<u8>
}

#[derive(Debug)]
pub struct AircarftPosition {
    raw_msg: [u8; 7],
    /// The altitude in feet
    altitude: u32,
    latitude: u32,
    longitude: u32
}

impl AircarftPosition {
    pub fn new(msg: [u8; 7]) -> Self {
        let alt_mode_25: bool = msg[1] & (1 << 0) == 1;
        let mut altitude = (((msg[1] >> 1) as u32) << 4) | ((((msg[2]) & 0xF0) as u32) >> 4); 

        if alt_mode_25 {altitude *= 25}
        else {altitude *= 100};

        altitude = altitude.saturating_sub(1000); 
        AircarftPosition { raw_msg: msg, altitude: altitude, latitude: 0, longitude: 0 }
    }
}

impl AdsbMsg for AircarftPosition {
    fn msg_id_match(id: u8) -> bool {
        9 <= id && id <= 18 
    }
}

#[derive(Debug)] 
pub struct AircraftID {
    raw_msg: [u8; 7],
    callsign: String
}

fn to_6bit_chunks(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;

    for byte in input.iter().copied() {
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

const CHAR_CONVERT: [char; 64] = [
    '#', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
    'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '#', '#', '#', '#', '#', '_', '#', '#', '#',
    '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '0', '1', '2', '3', '4', '5',
    '6', '7', '8', '9', '#', '#', '#', '#', '#', '#',
];

impl AircraftID {
    pub fn new(msg: [u8; 7]) -> Self {
        let msg_6_bit = to_6bit_chunks(&msg[1..msg.len()]);
        let mut callsign: String = String::new();

        for byte in msg_6_bit.iter() {
            if let Some(&ch) = CHAR_CONVERT.get(*byte as usize) {
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

impl AdsbMsg for AircraftID {
    /// Returns true if id can be parsed by this struct 
    fn msg_id_match(id: u8) -> bool {
        1 <= id && id <= 4 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aircraft_id() {
        let data: [u8; 7] = [0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0];

        let id = AircraftID::new(data);
        assert_eq!(id.callsign, "KLM1023_");
    }

    #[test]
    fn test_aircraft_position_25() {
        let data: [u8; 7] = [0x58, 0xC3, 0x82, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);
        assert_eq!(pos.altitude, 38000);
    }

    #[test]
    fn test_aircraft_position_100() {
        let data: [u8; 7] = [0x58, 0xC2, 0x82, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);
        assert_eq!(pos.altitude, 155000);
    }
}