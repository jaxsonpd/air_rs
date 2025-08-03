/// Module for calculating position from CPR (Compressed Position Reporting) data.
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use serde::Serialize;
use ts_rs::TS;

use crate::adsb::msgs::CprFormat;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GeographicPosition {
    pub latitude: f64,
    pub longitude: f64,
}


const NUM_ZONES: f64 = 15.0;

/// Convert a CPR value to a float
fn convert_cpr_to_float(cpr: u32) -> f64 {
    const CPR_TO_FLOAT: f64 = 131072.0; // 2^17
    (cpr as f64) / CPR_TO_FLOAT
}

fn normalize_longitude(mut lon: f64) -> f64 {
    while lon < -180.0 { lon += 360.0; }
    while lon > 180.0 { lon -= 360.0; }
    lon
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
/// 
/// `even_cpr_lat` - the even CPR latitude
/// `odd_cpr_lat` - the odd CPR latitude
/// `first` - the first CPR format recieved (oldest)
/// 
/// returns the selected latitude the even latitude and the odd latitude
fn calculate_latitude(even_cpr_lat: u32, odd_cpr_lat: u32, first: CprFormat) -> (f64, f64, f64){
    
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

    (latitude, even_latitude, odd_latitude)
    
}

fn calculate_longitude(even_cpr_long: u32, odd_cpr_long: u32, latitude: f64, first: CprFormat) -> f64 {
    let lon_cpr_e = convert_cpr_to_float(even_cpr_long);
    let lon_cpr_o = convert_cpr_to_float(odd_cpr_long);

    let num_zones: f64;

    let nl = calc_num_zones(latitude); 

    match first {
        CprFormat::Even => {
            // Later is odd
            num_zones = (calc_num_zones(latitude-1.0)).max(1) as f64
        },
        CprFormat::Odd => {
            // Later is even
            num_zones = (calc_num_zones(latitude)).max(1) as f64
        },
    }

    
    let divisions = 360.0 / num_zones;
    let m = (lon_cpr_e * ((nl - 1) as f64) - lon_cpr_o * (nl as f64) + 0.5).floor();

    let mut longitude: f64;

    match first {
        CprFormat::Even => {
            longitude = divisions * ((m % num_zones) + lon_cpr_o);
        },
        CprFormat::Odd => {
            longitude = divisions * ((m % num_zones) + lon_cpr_e);
        },
    }
     
    longitude = normalize_longitude(longitude);
    return longitude;
}

/// Calculate the geographic position from the even and odd CPR positions
/// 
/// `even_cpr_lat_long` - the latitude and longitude from the even CPR packet
/// `odd_cpr_lat_long` - the latitude and longitude from the odd CPR packet
/// `first` - the first CPR format (even or odd)
/// 
/// returns a GeographicPosition if it can be calculated, otherwise None
pub fn calculate_geographic_position(even_cpr_lat_long: (u32, u32), odd_cpr_lat_long: (u32, u32), first: CprFormat) -> Option<GeographicPosition> {
    let (latitude, even_latitude, odd_latitude) = calculate_latitude(even_cpr_lat_long.0, odd_cpr_lat_long.0, first);
    
    if calc_num_zones(even_latitude) != calc_num_zones(odd_latitude) {
        println!("Error: Number of zones for even and odd latitudes do not match");
        return None; 
    }
    
    let longitude = calculate_longitude(even_cpr_lat_long.1, odd_cpr_lat_long.1, latitude, first);


    Some(GeographicPosition { latitude: latitude, longitude: longitude })
}

mod tests {
    use super::*;

    #[test]
    fn test_latitude_calculation() {
        let even_cpr_lat = 93000; // Example even CPR latitude
        let odd_cpr_lat = 74158; // Example odd CPR latitude
        let first = CprFormat::Odd;

        let latitude = calculate_latitude(even_cpr_lat, odd_cpr_lat, first);
        assert!((latitude.0 - 52.25720).abs() < 0.0001); // Adjust expected value based on actual calculation
    }

    #[test]
    fn test_zone_calcuation() {
        assert_eq!(calc_num_zones(0.0), 59);

        assert_eq!(calc_num_zones(87.0), 2);

        assert_eq!(calc_num_zones(-87.0), 2);

        assert_eq!(calc_num_zones(90.0), 1);

        assert_eq!(calc_num_zones(-90.0), 1);

        assert_eq!(calc_num_zones(10.0), 59);

        assert_eq!(calc_num_zones(52.25720214843750), 36);
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
    fn test_identify_issue_with_latitude() {
        let even_cpr_lat = 23868; // Example even CPR latitude
        let odd_cpr_lat = 38688; // Example odd CPR latitude
        let first = CprFormat::Odd;

        let (latitude, even_latitude, odd_latitude) = calculate_latitude(even_cpr_lat, odd_cpr_lat, first);
        println!("Even Latitude: {}, Odd Latitude: {}, Calculated Latitude: {}", even_latitude, odd_latitude, latitude);

        assert_eq!(calc_num_zones(even_latitude), calc_num_zones(odd_latitude));

        let even_cpr_long = 111509; // Example even CPR longitude
        let odd_cpr_long = 47864; // Example odd CPR longitude
        let longitude = calculate_longitude(even_cpr_long, odd_cpr_long, latitude, CprFormat::Odd);
        
        println!("Longitude: {}", longitude);
        }
}