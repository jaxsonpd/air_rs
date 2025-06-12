# AIR_RS

This project aims to provide a suite of common applications utilising software defined radios (sdr).
This is intended to be an all in one program that allows the users to use
an sdr with minimal setup. THe key differentiator from programs like sdr++ being that you just start the program and go no plugin needed. 

Currently the program supports:

- ADS-B, stream, interactive tui and a web ui comming.
- 
The planned features are:

- Maritime Shipping 
- Weather sats NOAA etc. 

## Show off

Here is the current state of the project:

### ADSB Web GUI

![alt text](/doc/images/current_adsb_gui.png)

### ADSB Terminal Interface Interactive

to come.

### ADSB Terminal Interface 

to come.

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

These threads can then be swapped out based on the ui and processing methods needed.

## Reference Material

This project is based on several reference materials:

- https://github.com/rsadsb/dump1090_rs
- https://github.com/kevinmehall/rust-soapysdr
- https://github.com/ccostes/rtl-sdr-rs
