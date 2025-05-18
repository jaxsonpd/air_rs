use num_complex::Complex;
use soapysdr::{Direction, Device};
use clap::{Parser, Subcommand};

mod utils;
use utils::get_magnitude;

mod sdr;
use sdr::{get_sdr_args, list_devices};

mod adsb;
use adsb::{check_preamble, check_df, extract_packet, print_raw_packet, print_raw_buf};

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

fn launch_adsb(device: Option<u32>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Launching adsb with device: {:?}", device);
    // Find RTL-SDR device
    let args = get_sdr_args(device)?;

    let dev = Device::new(args)?;

    dev.set_gain_element(Direction::Rx, SDR_CHANNEL, "TUNER", SDR_GAIN)?;

    dev.set_frequency(Direction::Rx, SDR_CHANNEL, 1_090_000_000.0, ())?;

    dev.set_sample_rate(Direction::Rx, SDR_CHANNEL, 2_000_000.0)?;
    println!("Set up sdr device to 1090MHz freq and 2MHz sample");

    let mut stream = dev.rx_stream::<Complex<i16>>(&[SDR_CHANNEL])?;
    let mut buf = vec![Complex::new(0, 0); stream.mtu()?];

    stream.activate(None)?;

    loop {
        match stream.read(&mut [&mut buf], 2_000_000) {
            Ok(len) => {
                let buf = &buf[0..len];
                println!("Got buffer: {}", buf.len());

                let mag_vec: Vec<u32> = get_magnitude(buf);

                let mut i = 0;
                while i < mag_vec.len()-112*2 {
                    if let Some((high, signal_power, noise_power)) = check_preamble(mag_vec[i..i+16].to_vec()) {
                        if check_df(mag_vec[i+16..i+16+10].to_vec()) {
                            println!("f i: {}, h: {}, s: {}, n {}", i, high, signal_power, noise_power);
                            if let Some(packet) = extract_packet(mag_vec[i+16..i+16+112].to_vec(), high) {
                                print_raw_buf(mag_vec[i+16..i+16+112].to_vec(), high);
                                print_raw_packet(packet);
                            
                                i += 16+112;
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
        println!("Loop");
    }
    Ok(())

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::parse();

    match cli.command {
        Commands::List => list_devices(),
        Commands::Adsb {device} => launch_adsb(device),
    }?;

    Ok(())
}
