/// A thread to receive data and save it to a file

use num_complex::Complex;

use soapysdr::{Device, Direction};

use chrono::Local;

use crate::cli;
use crate::sdr::{get_sdr_args};
use crate::utils::save_data;

const SDR_CHANNEL: usize = 0;

pub fn launch_receive(device: Option<u32>, cli_args: cli::ReceiveArgs) {
    let args = get_sdr_args(device).expect("Couldn't get sdr args");

    let dev = Device::new(args).expect("Couldn't create sdr device");

    dev.set_gain_element(Direction::Rx, SDR_CHANNEL, "TUNER", cli_args.gain).expect("Couldn't set gain");

    dev.set_frequency(Direction::Rx, SDR_CHANNEL, cli_args.frequency, ()).expect("Couldn't set frequency");

    dev.set_sample_rate(Direction::Rx, SDR_CHANNEL, cli_args.sample_rate).expect("couldn't set sample rate");
    println!("Set up sdr device to 1090MHz freq and 2MHz sample");

    let mut stream = dev.rx_stream::<Complex<i16>>(&[SDR_CHANNEL]).expect("Couldn't start stream");

    stream.activate(None).expect("Couldn't activate stream");

    let mut buf: Vec<Complex<i16>> = vec![Complex::new(0, 0); stream.mtu().expect("Couldn't get buf")];
    let mut data: Vec<Complex<i16>> = Vec::new();

    let start_time = Local::now();
    while (Local::now() - start_time) < chrono::TimeDelta::new(cli_args.period as i64, 0).expect("Couldn't create time delta reference") {
        match stream.read(&mut [&mut buf], 2_000_000) {
            Ok(len) => {
                let mut buf = buf[0..len].to_vec();
                data.append(&mut buf);
            }
            Err(_e) => continue,
        }
    }

    save_data(data.as_slice());
}