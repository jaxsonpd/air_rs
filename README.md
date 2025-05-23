# SDR Interface

This project aims to provide an interface to communicate with sdr's to perform common functions easily.
This is intended to be an all in one program that allows the users to use
an sdr with minimal setup. THe key differentiator from programs like sdr++ being that you just start the program and go no plugin needed. 

The planned features are:

- ADS-B - Some what implemented
- Maritime Shipping 
- Weather sats NOAA etc. 

## Usage

The program can be run using:

```bash
cargo run help
```

from the root directory.

## Architecture

### ADS-B

The main rust program works using three threads:

1. A thread that handles receiving data from the sdr.
2. A thread that handles converting that data from complex frequency values to bits then into the adsbpacket struct.
3. The display thread which serves the data to the user using several different methods.

## Reference Material

This project is based on several reference materials:

- https://github.com/rsadsb/dump1090_rs
- https://github.com/kevinmehall/rust-soapysdr
- https://github.com/ccostes/rtl-sdr-rs