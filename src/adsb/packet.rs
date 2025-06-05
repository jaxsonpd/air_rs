/// Handle storage of adsb packets
/// 
/// Author Jack Duignan (JackpDuignan@gmail.com)

use chrono::Local;

use crate::adsb::msgs::{AdsbMsg, AdsbMsgType, AircarftPosition, AircraftID, UknownMsg};

#[derive(Debug, Clone)]
pub struct AdsbPacket {
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
    pub fn new(packet: Vec<u8>) -> AdsbPacket{
        
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
            packet: packet,
            downlink_format: downlink_format,
            capability: capability,
            icao: icao,
            msg_type: msg_type,
            msg: msg,
            time_processed: Local::now()
        }
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