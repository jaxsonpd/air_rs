use std::ops::Add;

/// Module for holding aircraft information for use with displaying functionality
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)
use chrono::{Local, TimeZone};
use clap::Id;

use crate::adsb_msgs::AdsbMsgType;
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
        last_contact: Local.timestamp_opt(0, 0).single().expect("Time Stamp Creation didn't work") }
    }

    pub fn handle_packet(&mut self, msg: adsb::AdsbPacket) {
        if msg.icao != self.icao {
            return;
        }
        
        match msg.msg {
            AdsbMsgType::AircarftPosition(pos) => {
                self.altitude = pos.altitude;
                self.last_contact = msg.time_processed;
            }
            AdsbMsgType::AircraftID(id) => {
                self.callsign = id.callsign;
            }
            AdsbMsgType::Uknown(_unkown) => {
                return;
            }
        }
    }
}
