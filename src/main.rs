use num_complex::Complex;
use soapysdr::{Direction, Device};
use clap::{Parser, Subcommand};

use std::thread;
use std::time::Duration;

use std::sync::mpsc::{self, Sender, Receiver};

mod utils;
use utils::get_magnitude;

mod sdr;
use sdr::{get_sdr_args, list_devices};

mod adsb;
use adsb::{check_df, check_preamble, extract_manchester, AdsbPacket};

mod adsb_msgs;

mod aircraft;

const SDR_GAIN: f64 = 49.50;
const SDR_CHANNEL: usize = 0;

#[derive(Parser, Debug)]
#[command(name = "sdr-interface")]
#[command(about = "Tool to interface with sdr devices", long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand, Debug)]
enum Commands {
    List,
    Adsb {
        #[arg(short, long)]
        device: Option<u32>,
    }
}

fn launch_adsb(device: Option<u32>) {
    println!("Launching adsb with device: {:?}", device);
    // Find RTL-SDR device
    let args = get_sdr_args(device).expect("Couldn't get sdr args");

    let dev = Device::new(args).expect("Couldn't create sdr device");

    dev.set_gain_element(Direction::Rx, SDR_CHANNEL, "TUNER", SDR_GAIN).expect("Couldn't set gain");

    dev.set_frequency(Direction::Rx, SDR_CHANNEL, 1_090_000_000.0, ()).expect("Couldn't set frequency");

    dev.set_sample_rate(Direction::Rx, SDR_CHANNEL, 2_000_000.0).expect("couldn't set sample rate");
    println!("Set up sdr device to 1090MHz freq and 2MHz sample");

    let (tx_raw_sdr, rx_raw_sdr): (Sender<Vec<Complex<i16>>>, Receiver<Vec<Complex<i16>>>) = mpsc::channel();

    let stream_thread = thread::spawn( move || {
        let mut stream = dev.rx_stream::<Complex<i16>>(&[SDR_CHANNEL]).expect("Couldn't start stream");

        stream.activate(None).expect("Couldn't activate stream");

        let mut buf: Vec<Complex<i16>> = vec![Complex::new(0, 0); stream.mtu().expect("Could get buf")];
        
        loop {
            match stream.read(&mut [&mut buf], 2_000_000) {
                Ok(len) => {
                    let buf = buf[0..len].to_vec();
                    if tx_raw_sdr.send(buf).is_err() {
                        println!("Raw sdr receiver is dropped");
                        break;
                    }
                }
                Err(_e) => continue,
            }
        }
    });

    let (tx_adsb_msgs, rx_adsb_msgs):(Sender<AdsbPacket>, Receiver<AdsbPacket>) = mpsc::channel();

    let process_thread = thread::spawn(move || {
        while let Ok(buf) = rx_raw_sdr.recv() {
            let mag_vec: Vec<u32> = get_magnitude(&buf); // Accepts &[Complex<i16>]

            let mut i = 0;
            while i < (mag_vec.len() - (16 + 112 * 2)) {
                if let Some((high, _signal_power, _noise_power)) = check_preamble(mag_vec[i..i + 16].to_vec()) {
                    if check_df(mag_vec[i + 16..i + 16 + 10].to_vec()) {
                        if let Some(raw_buf) = extract_manchester(mag_vec[i + 16..i + 16 + 112 * 2].to_vec(), high) {
                            let packet = AdsbPacket::new(raw_buf);
                            if tx_adsb_msgs.send(packet).is_err() {
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
    });

    let display_thread;
    if true {
        display_thread = thread::spawn(move || {
            loop {
                let mut current_aircraft: Vec<aircraft::Aircraft> = Vec::new();

                while let Ok(msg) = rx_adsb_msgs.try_recv() {
                    let mut handled = false;
                    for plane in current_aircraft.iter_mut() {
                        if plane.get_icao() == msg.get_icao() {
                            plane.handle_packet(msg.clone());
                            handled = true;
                            break;
                        }
                    }
                    if !handled {
                        current_aircraft.push(aircraft::Aircraft::new(msg.icao));
                        let current_aicraft_len = current_aircraft.len();
                        current_aircraft[current_aicraft_len].handle_packet(msg.clone());
                    }
                }
                print!("\x1B[2J\x1B[1;1H");
                println!("  icao  | Callsign   | Altitude | Age |");
                println!("----------------------------------------");
                for plane in current_aircraft.iter() {
                    println!(
                        " {:06} | {:<10} |  {:06}  | {:03} |",
                        plane.get_icao(),
                        plane.get_callsign(),
                        plane.get_altitude_ft(),
                        plane.get_age()
                    );
                }

                current_aircraft.retain(|a| a.get_age() <= 30);

                thread::sleep(Duration::from_secs(1));
            }
        });
    } else {
        display_thread = thread::spawn(move || {
            while let Ok(packet) = rx_adsb_msgs.recv() {
                print!("{}", packet);
            }
        });
    }

    let _ = stream_thread.join();
    let _ = process_thread.join();
    let _ = display_thread.join();
}

fn main() {
    let cli = CliArgs::parse();

    match cli.command {
        Commands::List => list_devices().expect("Couldn't start sdr sub process"),
        Commands::Adsb {device} => launch_adsb(device),
    };
}
