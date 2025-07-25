/// Implementation for the adsb packet structure and handling

use std::thread;
use std::time::Duration;

use std::sync::mpsc::{self, Sender, Receiver};

use num_complex::Complex;
use soapysdr::{Device, Direction};

mod aircraft;
mod tui;
mod msgs;
mod packet;
mod demod;
mod crc;

use packet::AdsbPacket;

use crate::cli::DisplayMode;
use crate::sdr::get_sdr_args;
use crate::utils::{get_magnitude, load_data};

const SDR_GAIN: f64 = 49.50;
const SDR_CHANNEL: usize = 0;

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
                    return;
                }
            }
            Err(_e) => continue,
        }
    }
}

fn playback_thread(tx: Sender<Vec<Complex<i16>>>, data: Vec<Complex<i16>>) {
    let mut i: usize = 0;
    while i < data.len()-20000 {
        let buf = data[i..i+20000].to_vec();
        i += 20000;
        if tx.send(buf).is_err() {
            println!("Raw sdr receiver is dropped");
            return;
        }
        thread::sleep(Duration::from_secs_f64(1e4/2e6));
        
    }

    drop(tx);
}

/// Process incoming sdr data sending the result to the display queue
fn process_sdr_data_thread(rx: Receiver<Vec<Complex<i16>>>, tx: Sender<AdsbPacket>) {
    let mut num_good = 0;
    let mut num_processed = 0;
    while let Ok(buf) = rx.recv() {
        let mags: Vec<u32> = get_magnitude(&buf); // Accepts &[Complex<i16>]
        
        for mut _i in 0..(mags.len() - (16 + 112 * 2)) {
            let check_mags: [u32; 32] =  mags[_i.._i + 32]
                            .try_into()
                            .expect("Bad packet length passed to adsb checker");
            
            if let Some((high, _signal_power, _noise_power)) 
                    = demod::check_for_adsb_packet(check_mags) {
                num_processed += 1;
                if let Some(packet_buf) = demod::extract_packet(mags[_i+16.._i+112*2+16].to_vec(), high) {
                    let packet = AdsbPacket::new(packet_buf);
                    if tx.send(packet).is_err() {
                        println!("Adsb msg receiver is dropped");
                        return;
                    }
                    num_good += 1;
                    _i += 16 + 112 * 2;
                }
            }
        }

        
    }
    println!("Processed: {}, Good: {}", num_processed, num_good);
    drop(tx);
}



pub fn launch_adsb(device: Option<u32>, mode: DisplayMode, playback: Option<String>) {
    println!("Launching adsb with device: {:?}", device);
    // Find RTL-SDR device
    

    let (tx_raw_sdr, rx_raw_sdr): (Sender<Vec<Complex<i16>>>, Receiver<Vec<Complex<i16>>>) = mpsc::channel();
    let _stream_thread;
    if playback.is_some() {
        println!("Starting data load from playback file: {}", playback.as_ref().unwrap());
        let data = load_data(playback.unwrap()).expect("Couldn't load playback data file");
        println!("Loaded {} samples from playback file", data.len());
        
        _stream_thread = thread::spawn(move || {
            playback_thread(tx_raw_sdr, data);
        });
    } else {
        let dev = setup_sdr(device);
        _stream_thread = thread::spawn(move || {get_sdr_data_thread(dev, tx_raw_sdr);});
    }
    let (tx_adsb_msgs, rx_adsb_msgs):(Sender<AdsbPacket>, Receiver<AdsbPacket>) = mpsc::channel();
    let _process_thread = thread::spawn(move || {process_sdr_data_thread(rx_raw_sdr, tx_adsb_msgs);});

    let display_thread;
    match mode {
        DisplayMode::Interactive => {
            display_thread = thread::spawn(move || {tui::interactive_display_thread_tui(rx_adsb_msgs);});
        },
        DisplayMode::Stream => {
            display_thread = thread::spawn(move || {
                while let Ok(packet) = rx_adsb_msgs.recv() {
                    print!("\n{}\n", packet);
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