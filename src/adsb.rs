/// Implementation for the adsb packet structure and handling

use chrono::Local;

use std::thread;
use std::time::Duration;

use std::sync::mpsc::{self, Sender, Receiver};

use num_complex::Complex;
use soapysdr::{Device, Direction};

mod aircraft;
mod tui;
mod msgs;

use aircraft::Aircraft;
use msgs::{AdsbMsgType, AircraftID, UknownMsg, AircarftPosition, AdsbMsg};

use crate::cli::DisplayMode;
use crate::sdr::get_sdr_args;
use crate::utils::{get_magnitude, load_data};

const SDR_GAIN: f64 = 49.50;
const SDR_CHANNEL: usize = 0;

#[derive(Debug, Clone)]
pub struct AdsbPacket {
    _raw_manchester: Vec<u16>,
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
            _raw_manchester: raw_buf,
            _raw_manchester: raw_buf,
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


/// Setup the sdr device with the correct values
/// 
/// device - the device number to use
/// 
/// returns the sdr device for use
fn setup_sdr(device: Option<u32>) -> Device {
    let args = get_sdr_args(device).expect("Couldn't get sdr args");

    let dev = Device::new(args).expect("Couldn't create sdr device");

    dev.set_gain_element(Direction::Rx, SDR_CHANNEL, "TUNER", SDR_GAIN).expect("Couldn't set gain");

    dev.set_frequency(Direction::Rx, SDR_CHANNEL, 1_090_000_000.0, ()).expect("Couldn't set frequency");

    dev.set_sample_rate(Direction::Rx, SDR_CHANNEL, 2_000_000.0).expect("couldn't set sample rate");
    println!("Set up sdr device to 1090MHz freq and 2MHz sample");

    dev
}

/// Get data from the sdr and then send it to a thread to process it
/// 
/// dev - the device to get data from
/// tx - the tx handler to use to send the data
fn get_sdr_data_thread(dev: Device, tx: Sender<Vec<Complex<i16>>>) {
    let mut stream = dev.rx_stream::<Complex<i16>>(&[SDR_CHANNEL]).expect("Couldn't start stream");

    stream.activate(None).expect("Couldn't activate stream");

    let mut buf: Vec<Complex<i16>> = vec![Complex::new(0, 0); stream.mtu().expect("Couldn't get buf")];

    loop {
        match stream.read(&mut [&mut buf], 2_000_000) {
            Ok(len) => {
                let buf = buf[0..len].to_vec();
                if tx.send(buf).is_err() {
                    println!("Raw sdr receiver is dropped");
                    break;
                }
            }
            Err(_e) => continue,
        }
    }
}

fn playback_thread(tx: Sender<Vec<Complex<i16>>>, filename: String) {
    let data = load_data(filename).expect("Couldn't load playback data file");
    let mut i: usize = 0;
    loop {
        let buf = data[i..i+10000].to_vec();
        i += 10000;
        if tx.send(buf).is_err() {
            println!("Raw sdr receiver is dropped");
            break;
        }
        thread::sleep(Duration::from_secs_f64(1e4/2e6));
        
    }
}

/// Process incoming sdr data sending the result to the display queue
fn process_sdr_data_thread(rx: Receiver<Vec<Complex<i16>>>, tx: Sender<AdsbPacket>) {
    while let Ok(buf) = rx.recv() {
        let mag_vec: Vec<u32> = get_magnitude(&buf); // Accepts &[Complex<i16>]

        let mut i = 0;
        while i < (mag_vec.len() - (16 + 112 * 2)) {
            if let Some((high, _signal_power, _noise_power)) = check_preamble(mag_vec[i..i + 16].to_vec()) {
                if check_df(mag_vec[i + 16..i + 16 + 10].to_vec()) {
                    if let Some(raw_buf) = extract_manchester(mag_vec[i + 16..i + 16 + 112 * 2].to_vec(), high) {
                        let packet = AdsbPacket::new(raw_buf);
                        if tx.send(packet).is_err() {
                            println!("Adsb msg receiver is dropped");
                            break;
                        }
                        i += 16 + 112 * 2;
                        continue;
                    }
                }
            }
            i += 1;
        }
    }
}

/// Display recivied packets in an interactive table format
fn interactive_display_thread(rx: Receiver<AdsbPacket>) {
    let mut current_aircraft: Vec<Aircraft> = Vec::new();

    loop {

        while let Ok(msg) = rx.try_recv() {
            let mut handled = false;
            for plane in current_aircraft.iter_mut() {
                if plane.get_icao() == msg.get_icao() {
                    plane.handle_packet(msg.clone());
                    handled = true;
                    break;
                }
            }
            if !handled {
                current_aircraft.push(Aircraft::new(msg.icao));
                let current_aicraft_len = current_aircraft.len();
                current_aircraft[current_aicraft_len-1].handle_packet(msg.clone());
            }
        }
        print!("\x1B[2J\x1B[1;1H");
        println!("  icao  | Callsign   | Altitude | Age |");
        println!("----------------------------------------");
        for plane in current_aircraft.iter() {
            println!(
                " {:06x} | {:<10} |  {:06}  | {:02}s |",
                plane.get_icao(),
                plane.get_callsign(),
                plane.get_altitude_ft(),
                plane.get_age()
            );
        }

        current_aircraft.retain(|a| a.get_age() <= 30);

        thread::sleep(Duration::from_secs(1));
    }
}



pub fn launch_adsb(device: Option<u32>, mode: DisplayMode, playback: Option<String>) {
    println!("Launching adsb with device: {:?}", device);
    // Find RTL-SDR device
    

    let (tx_raw_sdr, rx_raw_sdr): (Sender<Vec<Complex<i16>>>, Receiver<Vec<Complex<i16>>>) = mpsc::channel();
    let stream_thread;
    if playback.is_some() {
        stream_thread = thread::spawn(move || {
            playback_thread(tx_raw_sdr, playback.unwrap());
        });
    } else {
        let dev = setup_sdr(device);
        stream_thread = thread::spawn(move || {get_sdr_data_thread(dev, tx_raw_sdr);});
    }
    let (tx_adsb_msgs, rx_adsb_msgs):(Sender<AdsbPacket>, Receiver<AdsbPacket>) = mpsc::channel();
    let process_thread = thread::spawn(move || {process_sdr_data_thread(rx_raw_sdr, tx_adsb_msgs);});

    let display_thread;
    match mode {
        DisplayMode::Interactive => {
            display_thread = thread::spawn(move || {tui::interactive_display_thread_tui(rx_adsb_msgs);});
        },
        DisplayMode::Stream => {
            display_thread = thread::spawn(move || {
                while let Ok(packet) = rx_adsb_msgs.recv() {
                    print!("{}", packet);
                }
            });
        }
        DisplayMode::Web => {
            display_thread = thread::spawn(move || {
                println!("Web Display not implemented yet please restart");
            });
            
        }
    }

    // let _ = stream_thread.join();
    // let _ = process_thread.join();
    let _ = display_thread.join();

}