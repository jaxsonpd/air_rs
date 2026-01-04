/// Sub command to allow for receiving of aviation radio signals

use soapysdr::{Device, Direction};
use crate::sdr::get_sdr_args;

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

pub fn launch_com(device: Option<u32>, frequency: f64) {
    println!("Launching com with device: {:?}", device);

    let dev = setup_sdr(device);

}