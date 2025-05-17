use std::iter;

use plotters::prelude::*;
use num_complex::Complex;
use soapysdr::{Direction, Device, Args};
use clap::{Parser, Subcommand};
use chrono::Local;

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

fn list_devices() -> Result<(), Box<dyn std::error::Error>>{
    for (i, dev) in soapysdr::enumerate("")?.iter().enumerate() {
        println!("{}: {}, S/N: {}", i, dev.get("product").unwrap_or(dev.get("label").unwrap_or("N/A")), dev.get("serial").unwrap_or("N/A"));
    }

    Ok(())
}

fn get_sdr_args(device: Option<usize>) -> Result<soapysdr::Args, Box<dyn std::error::Error>> {
    // If device is not present default to rtlsdr
    let device_args = soapysdr::enumerate("")?;

    if let Some(device_num) = device {
        if device_args.len() < device_num {
            let args  = soapysdr::enumerate("").unwrap().remove(0);
            return Ok(args);
        } else {
            return Err("Device number is larger then all possible devices".into());
        }
    }

    let mut rtl_args: Option<Args> = None;

    for dev in soapysdr::enumerate("")? {
        println!("Enumerated: {}", dev);
        if dev.get("driver").unwrap_or("") == "rtlsdr" {
            rtl_args = Some(dev);
            break;
        }
    }

    let args = match rtl_args {
        Some(args) => {
            args
        }
        None => {
            return Err("Could not find RTL-SDR device consider defining explicit device number".into());
        }
    };

    Ok(args)
}

fn get_magnitude(buf: &[Complex<i16>]) -> Vec<i32> {
    buf.iter()
        .map(|c| ((c.re as f64).powi(2) + (c.im as f64).powi(2)).sqrt() as i32)
        .collect()
}

fn plot_adsb_frame(buf: Vec<i32>) {
    // Plot magnitude
    let filename = format!("magnitude_plot_{}.png", Local::now());
    let root = BitMapBackend::new(&filename, (1024, 768)).into_drawing_area();

    root.fill(&WHITE);

    // Compute magnitudes and store them as i32 values
    let min_val = *buf.iter().min().unwrap_or(&0);
    let max_val = *buf.iter().max().unwrap_or(&1);

    let mut chart = ChartBuilder::on(&root)
    .caption("Magnitude of SDR Samples", ("sans-serif", 30))
    .margin(20)
    .x_label_area_size(30)
    .y_label_area_size(40)
    .build_cartesian_2d(0..buf.len() as usize, min_val..max_val).unwrap();

    chart.configure_mesh().draw();

    chart
    .draw_series(LineSeries::new(
        buf.iter().enumerate().map(|(i, &m)| (i, m)),
        &BLUE,
    )).unwrap()
    .label("Magnitude")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    root.present();
    println!("Plot saved to {}", filename);
}

fn check_preamble(buf: Vec<i32>) -> Option<(i32, i32, i32)> {
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


    Some(((min as f32 * 0.9) as i32, 0, 0))
}

fn check_df(buf: Vec<i32>) -> bool {
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

/// Extract the contents of a packet
/// 
/// buf - the data buffer
/// high - the high level threshold
fn extract_packet(buf: Vec<i32>, high: i32) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut inter: u8 = 0;
    for i in (0..112*2).step_by(2) {
        if buf[i] > high && buf[i+1] < high { // 1
            inter |= 1 << (7 - (i % 8));
        }
        if i % 16 == 0 && i != 0{
            result.push(inter);
            inter = 0;
        }
    }

    result
}

/// Print out a preamble to allow for checking of it
/// 
fn print_preamble(buf: Vec<i32>) {
    for val in buf {
            print!(" {:^5} ", val);
    }

    print!("\n");

    for val in 0..16 {
        print!(" {:^5} ", val);
    }
    print!("\n");


} 

fn print_preamble_graph(buf: Vec<i32>) {
    let mut changed_buf: Vec<i32> = Vec::new();
    let max_val = buf.iter().max().unwrap();
    
    // for i in 0..16 {
    //     changed_buf = 3

    // }
    print!("\u{2581}");
}

fn launch_adsb(device: Option<u32>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Launching adsb with device: {:?}", device);
    // Find RTL-SDR device
    let args = get_sdr_args(None)?;

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

            let magnitudes: Vec<i32> = get_magnitude(buf);
            let mut i = 0;
            while i < magnitudes.len()-112*2 {
                if let Some((high, signal_power, noise_power)) = check_preamble(magnitudes[i..i+16].to_vec()) {
                    if check_df(magnitudes[i+16..i+16+10].to_vec()) {
                        println!("f i: {}, h: {}, s: {}, n {}", i, high, signal_power, noise_power);
                        // print_preamble(magnitudes[i..i+16].to_vec());
                        // print_preamble_graph(magnitudes[i..i+16].to_vec());
                        let msg = extract_packet(magnitudes[i+15..i+16+112*2].to_vec(), high);
                        for byte in msg.iter() {
                            print!("{:08b} ", byte);
                            print!("{:02x} ", byte);
                        }
                        plot_adsb_frame(magnitudes[i..i+50].to_vec());
                        i+=16+112*2;
                    } else {
                        i+=1;
                    }
                } else {
                    i+=1;
                }
            }
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
