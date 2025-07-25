/// Module for holding aircraft information for use with displaying functionality
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use chrono::{Local};

use crate::adsb::msgs::{AdsbMsgType, CprFormat};
use crate::adsb;

#[derive(Debug, Clone)]
pub struct GeographicPosition {
    pub latitude: f64,
    pub longitude: f64,
}

/// Holder for aircraft information that has been received from adsb
#[derive(Debug)]
pub struct Aircraft {
    icao: u32,
    callsign: Option<String>,
    altitude: i32,
    geo_position: Option<GeographicPosition>,
    last_contact: chrono::prelude::DateTime<Local>,
    last_odd_packet: Option<adsb::msgs::AircraftPosition>,
    last_even_packet: Option<adsb::msgs::AircraftPosition>,
}

impl Aircraft {
    pub fn new(icao: u32) -> Self {
        Aircraft { icao: icao, callsign: None, 
        altitude: 0, geo_position: None,
        last_contact: Local::now(), 
        last_odd_packet: None, last_even_packet: None } 
        }

    pub fn handle_packet(&mut self, msg: adsb::AdsbPacket) {
        if msg.get_icao() != self.icao {
            return;
        }

        match msg.msg {
            AdsbMsgType::AircraftPosition(ref pos) => {
                self.altitude = pos.get_altitude_ft();
                self.last_contact = msg.time_processed;
                match pos.get_cpr_format() {
                    adsb::msgs::CprFormat::Even => {
                        if let Some(odd_pos) = &self.last_odd_packet {
                            let odd_cpr_lat_long = odd_pos.get_cpr_position();
                            let even_cpr_lat_long = pos.get_cpr_position();
                            
                            self.geo_position = Some(calculate_geographic_position(even_cpr_lat_long, odd_cpr_lat_long, CprFormat::Odd));
                        }
                        self.last_even_packet = Some(pos.clone());
                    },
                    adsb::msgs::CprFormat::Odd => {
                        if let Some(even_pos) = &self.last_even_packet {
                            let odd_cpr_lat_long = pos.get_cpr_position();
                            let even_cpr_lat_long = even_pos.get_cpr_position();
                            
                            self.geo_position = Some(calculate_geographic_position(even_cpr_lat_long, odd_cpr_lat_long, CprFormat::Even));
                        }  
                        self.last_odd_packet = Some(pos.clone());
                    },
                }
                
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


const NUM_ZONES: f64 = 15.0;

/// Convert a CPR value to a float
fn convert_cpr_to_float(cpr: u32) -> f64 {
    const CPR_TO_FLOAT: f64 = 131072.0; // 2^17
    (cpr as f64) / CPR_TO_FLOAT
}

/// Find the number of longitude zones for a given latitude
/// 
/// `lat` - the latitude in degrees
/// 
/// returns the number of longitude zones
fn calc_num_zones(lat: f64) -> u32 {
    if lat == 0.0 {
        return 59; // Special case for equator
    } else if lat == 87.0 || lat == -87.0 {
        return 2; // Special case for poles
    } else if lat < -87.0 || lat > 87.0 {
        return 1; // Invalid latitude
    }

    let pi = std::f64::consts::PI;
    let int1 = 1.0 - (pi/(2.0*NUM_ZONES)).cos();
    let int2 = (pi/180.0 * lat).cos();
    let int3 = (2.0 * pi) / (1.0 - (int1/(int2*int2))).acos();

    int3.floor() as u32
}

/// Calculate the latitude from the even and odd CPR latitudes
fn calculate_latitude(even_cpr_lat: u32, odd_cpr_lat: u32, first: CprFormat) -> f64{
    
    const EVEN_LAT_DIVISIONS: f64 = 360.0 / (4.0 * NUM_ZONES);
    const ODD_LAT_DIVISIONS: f64 = 360.0 / (4.0 * NUM_ZONES-1.0);

    let even_cpr_lat = convert_cpr_to_float(even_cpr_lat);
    let odd_cpr_lat = convert_cpr_to_float(odd_cpr_lat);

    let latitude_index: f64 = (59.0*even_cpr_lat - 60.0 * odd_cpr_lat + 0.5).floor();

    let even_latitude = EVEN_LAT_DIVISIONS * (latitude_index % 60.0 + even_cpr_lat);
    let odd_latitude = ODD_LAT_DIVISIONS * (latitude_index % 59.0 + odd_cpr_lat);

    let mut latitude = match first {
        // Use the newest format to determine the latitude
        CprFormat::Even => odd_latitude,
        CprFormat::Odd => even_latitude,
    };

    if latitude > 270.0 {
        latitude -= 360.0;
    }

    latitude
    
}

fn calculate_longitude(even_cpr_long: u32, odd_cpr_long: u32, latitude: f64, first: CprFormat) -> f64 {
    let lon_cpr_e = convert_cpr_to_float(even_cpr_long);
    let lon_cpr_o = convert_cpr_to_float(odd_cpr_long);

    let num_zones: f64;

    match first {
        CprFormat::Even => {
            // Later is odd
            num_zones = (calc_num_zones(latitude)-1).max(1) as f64
        },
        CprFormat::Odd => {
            // Later is even
            num_zones = (calc_num_zones(latitude)).max(1) as f64
        },
        
    }

    
    let divisions = 360.0 / num_zones;
    let m = (lon_cpr_e * (num_zones - 1.0) - lon_cpr_o * num_zones + 0.5).floor();
    let mut longitude = divisions * ((m % num_zones) + lon_cpr_o);

    if longitude > 180.0 {
        longitude -= 360.0;
    }

    longitude
}

/// Calculate the geographic position from the even and odd CPR positions
/// 
/// `even_cpr_lat_long` - the latitude and longitude from the even CPR packet
/// `odd_cpr_lat_long` - the latitude and longitude from the odd CPR packet
/// `first` - the first CPR format (even or odd)
/// 
/// returns a GeographicPosition
fn calculate_geographic_position(even_cpr_lat_long: (u32, u32), odd_cpr_lat_long: (u32, u32), first: CprFormat) -> GeographicPosition {
    let latitude = calculate_latitude(even_cpr_lat_long.0, odd_cpr_lat_long.0, first);
    let longitude = calculate_longitude(even_cpr_lat_long.1, odd_cpr_lat_long.1, latitude, first);


    GeographicPosition { latitude: latitude, longitude: longitude }
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
    fn test_latitude_calculation() {
        let even_cpr_lat = 93000; // Example even CPR latitude
        let odd_cpr_lat = 74158; // Example odd CPR latitude
        let first = CprFormat::Odd;

        let latitude = calculate_latitude(even_cpr_lat, odd_cpr_lat, first);
        assert!((latitude - 52.25720).abs() < 0.0001); // Adjust expected value based on actual calculation
    }

    #[test]
    fn test_zone_calcuation() {
        assert_eq!(calc_num_zones(0.0), 59);

        assert_eq!(calc_num_zones(87.0), 2);

        assert_eq!(calc_num_zones(-87.0), 2);

        assert_eq!(calc_num_zones(90.0), 1);

        assert_eq!(calc_num_zones(-90.0), 1);

        assert_eq!(calc_num_zones(10.0), 7);
    }

    #[test]
    fn test_longitude_calculation() {
        let even_cpr_long = 51372; // Example even CPR longitude
        let odd_cpr_long = 50194; // Example odd CPR longitude
        let latitude = 52.25720214843750; // Example latitude
        let first = CprFormat::Odd;

        let longitude = calculate_longitude(even_cpr_long, odd_cpr_long, latitude, first);
        assert!((longitude -  3.829498291015625).abs() < 0.0001); // Adjust expected value based on actual calculation
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
