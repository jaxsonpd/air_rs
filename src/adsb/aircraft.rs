/// Module for holding aircraft information for use with displaying functionality
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use chrono::{Local};

use crate::adsb::msgs::{AdsbMsgType, CprFormat};
use crate::adsb::{self, cpr};
use crate::adsb::cpr::{calculate_geographic_position, GeographicPosition};



/// Holder for aircraft information that has been received from adsb
#[derive(Debug, Clone)]
pub struct Aircraft {
    icao: u32,
    callsign: Option<String>,
    altitude: i32,
    geo_position: Option<GeographicPosition>,
    last_contact: chrono::prelude::DateTime<Local>,
    last_odd_packet: Option<adsb::msgs::AircraftPosition>,
    last_odd_processed: chrono::prelude::DateTime<Local>,
    last_even_packet: Option<adsb::msgs::AircraftPosition>,
    last_even_processed: chrono::prelude::DateTime<Local>,
}

impl Aircraft {
    pub fn new(icao: u32) -> Self {
        Aircraft { icao: icao, callsign: None, 
        altitude: 0, geo_position: None,
        last_contact: Local::now(), 
        last_odd_packet: None, last_even_packet: None,
        last_odd_processed: Local::now(), last_even_processed: Local::now() } 
        }

    pub fn handle_packet(&mut self, msg: adsb::AdsbPacket) {
        if msg.get_icao() != self.icao {
            return;
        }

        match msg.msg {
            AdsbMsgType::AircraftPosition(ref pos) => {
                self.altitude = pos.get_altitude_ft();
                self.last_contact = msg.time_processed;

                let cpr_odd;
                let cpr_even;
                let first;
                
                match pos.get_cpr_format() {
                    adsb::msgs::CprFormat::Even => {
                        self.last_even_packet = Some(pos.clone());
                        self.last_even_processed = msg.time_processed;

                        if let Some(odd_pos) = &self.last_odd_packet {
                            if (msg.time_processed - self.last_odd_processed).abs() > chrono::Duration::seconds(10) {
                                return;
                            }

                            cpr_odd = odd_pos.get_cpr_position();
                            cpr_even = pos.get_cpr_position();
                            first = CprFormat::Odd;
                        } else {
                            return;
                        }
                    },
                    adsb::msgs::CprFormat::Odd => {
                        self.last_odd_packet = Some(pos.clone());
                        self.last_odd_processed = msg.time_processed;

                        if let Some(even_pos) = &self.last_even_packet {
                            if (msg.time_processed - self.last_even_processed).abs() > chrono::Duration::seconds(10) {
                                return
                            }    

                            cpr_odd = pos.get_cpr_position();
                            cpr_even = even_pos.get_cpr_position();
                            first = CprFormat::Even;
                        }  else {
                            return;
                        }
                    },
                }

                if let Some(geo_position) = calculate_geographic_position(
                                                                cpr_even,
                                                                 cpr_odd, first) {
                    self.geo_position = Some(geo_position);
                };
                
            }
            AdsbMsgType::AircraftID(id) => {
                self.callsign = Some(id.get_callsign());
            }
            AdsbMsgType::Uknown(_unkown) => {
                return;
            }
        }
    }

    pub fn get_icao(&self) -> u32 {
        self.icao
    }

    pub fn get_callsign(&self) -> String {
        let callsign = match self.callsign.clone() {
            Some(callsign) => callsign,
            None => String::from(""),
        };

        callsign
    }

    pub fn get_altitude_ft(&self) -> i32 {
        self.altitude
    }

    /// Return the time since the last transmission in seconds
    pub fn get_age(&self) -> i64 {
        (chrono::Local::now() - self.last_contact).num_seconds()
    }

    pub fn get_geo_position(&self) -> Option<GeographicPosition> {
        self.geo_position.clone()
    }
}

/// Handle recieving aircraft packets and updating the aircraft information.
/// Adding an aircraft to the hashmap if it does not exist or updating it if it does.
/// 
/// 'packet' - the ADS-B packet to handle
/// 'aircrafts' - a hashmap of aircrafts to update
/// returns the aircraft that was updated or added
pub fn handle_aircraft_update(packet: adsb::AdsbPacket, aircrafts: &mut std::collections::HashMap<u32, Aircraft>) -> Option<Aircraft> {
    let icao = packet.get_icao();
    let aircraft = aircrafts.entry(icao).or_insert(Aircraft::new(icao));
    
    aircraft.handle_packet(packet);
    
    Some(aircraft.clone())
}

mod tests {
    #[allow(unused_imports)]
    use std::str::FromStr;
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::adsb::msgs::AdsbMsgType;
    #[allow(unused_imports)]
    use crate::adsb::AdsbPacket;

    #[test]
    fn test_aircraft_new() {
        let aircraft = Aircraft::new(0x123456);
        assert_eq!(aircraft.get_icao(), 0x123456);
        assert_eq!(aircraft.get_callsign(), "");
        assert_eq!(aircraft.get_altitude_ft(), 0);
    }

    #[test]
    fn test_aircraft_handle_packet_id() {
        let mut aircraft = Aircraft::new(0x7C6B30);
        let packet = AdsbPacket::_new_from_string(String::from_str("8d7c6b3020293532d70820fc8090").unwrap());
        aircraft.handle_packet(packet);
        assert_eq!(aircraft.get_callsign(), "JST250__");
    }

    #[test]
    fn test_aircraft_handle_packet_alt() {
        let mut aircraft = Aircraft::new(0x7C6B30);
        let packet = AdsbPacket::_new_from_string(String::from_str("8d7c6b30581304f388bb4455896f").unwrap());
        aircraft.handle_packet(packet);
        assert_eq!(aircraft.get_altitude_ft(), 2600);
    }

    #[test]
    fn test_aircraft_handle_packet_fake() {
        let mut aircraft = Aircraft::new(0x40621D);
        let first = AdsbPacket::_new_from_string(String::from_str("8D40621D58C386435CC412692AD6").unwrap());
        let second = AdsbPacket::_new_from_string(String::from_str("8D40621D58C382D690C8AC2863A7").unwrap());

        aircraft.handle_packet(first);
        aircraft.handle_packet(second);
        assert_eq!(aircraft.get_altitude_ft(), 38000);
        assert!((aircraft.geo_position.clone().unwrap().latitude - 52.25720).abs() < 0.0001);
        assert!((aircraft.geo_position.unwrap().longitude - 3.829498291015625).abs() < 0.0001);
    }

    #[test]
    fn test_aircraft_handle_packet_pos() {
        /*
        == 8d7c6b30580d107903b3cabf62ab ==
        Decoded Information:
        Downlink Format : 17
        Capability      : 5
        ICAO            : 7C6B30
        Processed Time  : 2025-07-26 07:47:16.818387100 +12:00
        Message Type    : 11
        Message:
        Type                : 11 (Position)
        Surveillance Status : 0
        NIC Supplement      : 0
        Altitude (ft)       : 1425
        CPR Time            : 0
        CPR polarity        : 0
        Raw Latititude      : 15489
        Raw Longitude       : 111562


        == 8d7c6b30580d24eeaebb2dfea5bb ==
        Decoded Information:
        Downlink Format : 17
        Capability      : 5
        ICAO            : 7C6B30
        Processed Time  : 2025-07-26 07:47:18.197151200 +12:00
        Message Type    : 11
        Message:
        Type                : 11 (Position)
        Surveillance Status : 0
        NIC Supplement      : 0
        Altitude (ft)       : 1450
        CPR Time            : 0
        CPR polarity        : 1
        Raw Latititude      : 30551
        Raw Longitude       : 47917
         */
        let mut aircraft = Aircraft::new(0x7C6B30);
        let first = AdsbPacket::_new_from_string(String::from_str("8d7c6b30580d107903b3cabf62ab").unwrap());
        let second = AdsbPacket::_new_from_string(String::from_str("8d7c6b30580d24eeaebb2dfea5bb").unwrap());
        
        aircraft.handle_packet(first);
        aircraft.handle_packet(second);

        assert_eq!(aircraft.get_altitude_ft(), 1450);
        assert!((aircraft.geo_position.clone().unwrap().latitude - -41.28964698920816).abs() < 0.0001);
        assert!((aircraft.geo_position.unwrap().longitude - 174.80927207253197).abs() < 0.0001);
    }
}
