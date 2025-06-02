use clap::Parser;

mod utils;

mod adsb;
use adsb::{launch_adsb};

mod sdr;
use sdr::list_devices;

mod adsb_msgs;

mod aircraft;

mod cli;
use cli::{Commands, CliArgs};

mod receive;
use receive::launch_receive;

fn main() {
    let cli = CliArgs::parse();

    match cli.command {
        Commands::List => list_devices().expect("Couldn't start sdr sub process"),
        Commands::Adsb {device, mode, playback} => launch_adsb(device, mode, playback),
        Commands::Receive {device, args} => launch_receive(device, args),
    };
}
