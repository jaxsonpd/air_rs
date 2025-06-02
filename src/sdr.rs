use soapysdr::{Args};


pub fn list_devices() -> Result<(), Box<dyn std::error::Error>>{
    for (i, dev) in soapysdr::enumerate("")?.iter().enumerate() {
        println!("{}: {}, S/N: {}", i, dev.get("product").unwrap_or(dev.get("label").unwrap_or("N/A")), dev.get("serial").unwrap_or("N/A"));
    }

    Ok(())
}

pub fn get_sdr_args(device: Option<u32>) -> Result<soapysdr::Args, Box<dyn std::error::Error>> {
    // If device is not present default to rtlsdr
    let device_args = soapysdr::enumerate("")?;

    if let Some(device_num) = device {
        if device_args.len() < device_num as usize {
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