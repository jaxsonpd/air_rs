/// Definitions for the adsb message types
///
/// Jack Duignan (JackpDuignan@gmail.com)

/// Top level enum to hold various message types
#[derive(Debug, Clone)]
pub enum AdsbMsgType {
    AircraftID(AircraftID),
    AircarftPosition(AircarftPosition),
    Uknown(UknownMsg)
}

impl std::fmt::Display for AdsbMsgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdsbMsgType::AircraftID(id) =>
                write!(f, "{}", id),
            AdsbMsgType::AircarftPosition(pos) =>
                write!(f, "{}", pos),
            AdsbMsgType::Uknown(msg) =>
                write!(f, "{}", msg),
        }
    }
}

/// Top level trait for messages
pub trait AdsbMsg: std::fmt::Debug {
    fn msg_id_match(id: u8) -> bool where Self: Sized;
}

#[derive(Debug, Clone)]
pub struct UknownMsg {
    pub raw_msg: Vec<u8>
}

impl std::fmt::Display for UknownMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Message:")?;
        writeln!(f, "Type    : Unknown")?;
        writeln!(f, "Raw Msg :  {:?}", self.raw_msg)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AircarftPosition {
    _raw_msg: [u8; 7],
    msg_type: u8,
    surveillance_status: u8,
    nic_sup: u8,
    /// The altitude in feet
    pub altitude: i32,
    pub cpr_time: u8,
    pub cpr_flag: u8, // odd = 1
    pub cpr_latitude: u32,
    pub cpr_longitude: u32
}

impl AircarftPosition {
    pub fn new(msg: [u8; 7]) -> Self {
        let alt_mode_25: bool = msg[1] & (1 << 0) == 1;
        let mut altitude = (((msg[1] >> 1) as i32) << 4) | ((((msg[2]) & 0xF0) as i32) >> 4); 

        if alt_mode_25 {altitude *= 25}
        else {altitude *= 100};

        altitude -= 1000; 

        let msg_type = (msg[0] & 0b1111_1000) >> 3;
        let ss = (msg[0] & 0b110) >> 1;
        let nic = msg[0] & 0b1;
        let time = (msg[2] & 0b1000) >> 3;
        let oddity = (msg[2] & 0b0100) >> 2;

        let latitude = (((msg[2] & 0b11) as u32) << 15) | ((msg[3] as u32) << 7) | (((msg[4] & 0b1111_1110) >> 1) as u32); 
        let longitude = (((msg[4] & 0b1) as u32) << 16) | ((msg[5] as u32) << 8) | (msg[6] as u32); 

        
        AircarftPosition { _raw_msg: msg, msg_type: msg_type, surveillance_status: ss, 
            nic_sup: nic, cpr_time: time, cpr_flag: oddity, 
            altitude: altitude, cpr_latitude: latitude, cpr_longitude: longitude }
    }

    pub fn get_altitude_ft(&self) -> i32{
        self.altitude
    }
}

impl AdsbMsg for AircarftPosition {
    fn msg_id_match(id: u8) -> bool {
        9 <= id && id <= 18 
    }
}

impl std::fmt::Display for AircarftPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Message:")?;
        writeln!(f, "Type                : {} (Position)", self.msg_type)?;
        writeln!(f, "Surveillance Status : {}", self.surveillance_status)?;
        writeln!(f, "NIC Supplement      : {}", self.nic_sup)?;
        writeln!(f, "Altitude (ft)       : {}", self.altitude)?;
        writeln!(f, "CPR Time            : {}", self.cpr_time)?;
        writeln!(f, "CPR polarity        : {}", self.cpr_flag)?;
        writeln!(f, "Raw Latititude      : {}", self.cpr_latitude)?;
        writeln!(f, "Raw Longitude       : {}", self.cpr_longitude)?;

        Ok(())
    }
}

#[derive(Debug, Clone)] 
pub struct AircraftID {
    _raw_msg: [u8; 7],
    msg_type: u8,
    pub callsign: String
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

        let msg_type = (msg[0] & 0b1111_1000) >> 3;


        Self {
            _raw_msg: msg,
            msg_type: msg_type,
            callsign: callsign
        }

    }

    pub fn get_callsign(&self) -> String {
        self.callsign.clone()
    }
}

impl AdsbMsg for AircraftID {
    /// Returns true if id can be parsed by this struct 
    fn msg_id_match(id: u8) -> bool {
        1 <= id && id <= 4 
    }
}

impl std::fmt::Display for AircraftID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Message:")?;
        writeln!(f, "Type                : {} (ID)", self.msg_type)?;
        writeln!(f, "Callsign            : {}", self.callsign)?;

        Ok(())
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
    fn test_aircraft_type() {
        let data: [u8; 7] = [0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0];

        let id = AircraftID::new(data);
        assert_eq!(id.msg_type, 4);
    }

    #[test]
    fn test_aircraft_position_alt_25() {
        let data: [u8; 7] = [0x58, 0xC3, 0x82, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);
        assert_eq!(pos.altitude, 38000);
    }

    #[test]
    fn test_aircraft_position_alt_100() {
        let data: [u8; 7] = [0x58, 0xC2, 0x82, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);
        assert_eq!(pos.altitude, 155000);
    }

    #[test]
    fn test_aircraft_position_neg_alt_100() {
        let data: [u8; 7] = [0x58, 0x01, 0x02, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);
        assert_eq!(pos.altitude, -1000);
    }

    #[test]
    fn test_aircraft_position_neg_alt_100_1() {
        let data: [u8; 7] = [0x58, 0x01, 0x12, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);
        assert_eq!(pos.altitude, -975);
    }

    #[test]
    fn test_aircraft_position_flags_even_frame() {
        let data: [u8; 7] = [0x58, 0xC3, 0x82, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);

        assert_eq!(pos.msg_type, 11);
        assert_eq!(pos.surveillance_status, 0);
        assert_eq!(pos.nic_sup, 0);
        assert_eq!(pos.cpr_time, 0);
        assert_eq!(pos.cpr_flag, 0);
    }

    #[test]
    fn test_aircraft_position_flags_odd_frame() {
        let data: [u8; 7] = [0x58, 0xc3, 0x86, 0x43, 0x5c, 0xc4, 0x12];

        let pos = AircarftPosition::new(data);

        assert_eq!(pos.msg_type, 11);
        assert_eq!(pos.surveillance_status, 0);
        assert_eq!(pos.nic_sup, 0);
        assert_eq!(pos.cpr_time, 0);
        assert_eq!(pos.cpr_flag, 1);
    }

    #[test]
    fn test_aircraft_position_flags_even_pos() {
        let data: [u8; 7] = [0x58, 0xC3, 0x82, 0xD6, 0x90, 0xC8, 0xAC];

        let pos = AircarftPosition::new(data);

        assert_eq!(pos.cpr_latitude, 93000);
        assert_eq!(pos.cpr_longitude, 51372);
    }

    #[test]
    fn test_aircraft_position_odd_frame_pos() {
        let data: [u8; 7] = [0x58, 0xc3, 0x86, 0x43, 0x5c, 0xc4, 0x12];

        let pos = AircarftPosition::new(data);

        assert_eq!(pos.cpr_latitude, 74158);
        assert_eq!(pos.cpr_longitude, 50194);
    }
}