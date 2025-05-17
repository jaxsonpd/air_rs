use soapysdr::{Direction, Device, Args};

#[derive(Debug)]
pub struct SDRDevice {
    device: soapysdr::Device,
}

impl SDRDevice {
    pub fn new(device: Option<usize>) -> Self {
         // If device is not present default to rtlsdr
        let device_args = soapysdr::enumerate("")?;

        if let Some(device_num) = device {
            if device_args.len() < device_num {
                if let Some(args)  = soapysdr::enumerate("").unwrap().get(0) {
                    return Ok(args.clone())
                }
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
                return Err("Could not find RTL-SDR device consider defining expliset device number".into());
            }
        };

    
    }
}