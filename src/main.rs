use num_complex::Complex;
use soapysdr::{Direction, Device};
use clap::{Parser, Subcommand};

mod utils;
use utils::get_magnitude;

mod sdr;
use sdr::{get_sdr_args, list_devices};

mod adsb;
use adsb::{check_df, check_preamble, extract_manchester, AdsbPacket};

mod adsb_msgs;

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

    let mut stream = dev.rx_stream::<Complex<i16>>(&[SDR_CHANNEL]).expect("Couldn't start stream");
    let mut buf = vec![Complex::new(0, 0); stream.mtu().expect("Could get buf")];

    stream.activate(None).expect("Couldn't activate stream");

    loop {
        match stream.read(&mut [&mut buf], 2_000_000) {
            Ok(len) => {
                let buf = &buf[0..len];

                let mag_vec: Vec<u32> = get_magnitude(buf);

                let mut i = 0;
                while i < (mag_vec.len()-(16+112*2)) {
                    if let Some((high, _signal_power, _noise_power)) = check_preamble(mag_vec[i..i+16].to_vec()) {
                        if check_df(mag_vec[i+16..i+16+10].to_vec()) {
                            if let Some(raw_buf) = extract_manchester(mag_vec[i+16..i+16+112*2].to_vec(), high) {
                                let packet = AdsbPacket::new(raw_buf);
                                println!("{}", packet);
                                i += 16+112*2;
                            };
                        }
                    }
                    i += 1;
                    
                } 
                // let mut i = 0;
                // while i < magnitudes.len()-112*2 {
                //     if let Some((high, signal_power, noise_power)) = check_preamble(magnitudes[i..i+16].to_vec()) {
                //         if check_df(magnitudes[i+16..i+16+10].to_vec()) {
                //             println!("f i: {}, h: {}, s: {}, n {}", i, high, signal_power, noise_power);
                //             // print_preamble(magnitudes[i..i+16].to_vec());
                //             // print_preamble_graph(magnitudes[i..i+16].to_vec());
                //             let msg = extract_packet(magnitudes[i+15..i+16+112*2].to_vec(), high);
                //             for byte in msg.iter() {
                //                 print!("{:08b} ", byte);
                //                 print!("{:02x} ", byte);
                //             }
                //             plot_adsb_frame(magnitudes[i..i+50].to_vec());
                //             i+=16+112*2;
                //         } else {
                //             i+=1;
                //         }
                //     } else {
                //         i+=1;
                    // }
            }
            Err(e) => println!("Error reading stream: {}", e),
        }
    }
}

fn main() {
    let cli = CliArgs::parse();

    match cli.command {
        Commands::List => list_devices().expect("Couldn't start sdr sub process"),
        Commands::Adsb {device} => launch_adsb(device),
    };
}
