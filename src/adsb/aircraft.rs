/// Module for holding aircraft information for use with displaying functionality
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use chrono::{Local};

use crate::adsb::msgs::AdsbMsgType;
use crate::adsb;

/// Holder for aircraft information that has been received from adsb
#[derive(Debug)]
pub struct Aircraft {
    icao: u32,
    callsign: String,
    altitude: i32,
    latitude: f64,
    longitude: f64,
    last_contact: chrono::prelude::DateTime<Local>
}

impl Aircraft {
    pub fn new(icao: u32) -> Self {
        Aircraft { icao: icao, callsign: "".to_string(), 
        altitude: 0, latitude: 0.0, longitude: 0.0, 
        last_contact: Local::now() 
    }
    }

    pub fn handle_packet(&mut self, msg: adsb::AdsbPacket) {
        if msg.get_icao() != self.icao {
            return;
        }

        match msg.msg {
            AdsbMsgType::AircarftPosition(pos) => {
                self.altitude = pos.get_altitude_ft();
                self.last_contact = msg.time_processed;
            }
            AdsbMsgType::AircraftID(id) => {
                self.callsign = id.get_callsign();
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
        self.callsign.clone()
    }

    pub fn get_altitude_ft(&self) -> i32 {
        self.altitude
    }

    /// Return the time since the last transmission in seconds
    pub fn get_age(&self) -> i64 {
        (chrono::Local::now() - self.last_contact).num_seconds()
    }
}
