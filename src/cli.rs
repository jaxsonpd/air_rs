/// Implementation of the cli for the tool
use clap::{Parser, Subcommand, ValueEnum, Args};

#[derive(Parser, Debug)]
#[command(name = "sdr-interface")]
#[command(about = "Tool to interface with sdr devices", long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    List,
    Receive {
        /// Optional device ID (option flag, not in struct)
        #[arg(short, long)]
        device: Option<u32>,

        #[command(flatten)]
        args: ReceiveArgs,
    },
    Adsb {
        #[arg(short, long)]
        device: Option<u32>,

        #[arg(short = 'm', long = "mode", default_value_t = DisplayMode::Stream)]
        mode: DisplayMode,
    }
}

#[derive(Args, Debug)]
pub struct ReceiveArgs {
    /// Frequency in Hz
    pub frequency: f64,

    /// Sample rate in Hz
    pub sample_rate: f64,

    /// Game identifier or name
    pub gain: f64,

    /// Period in seconds
    pub period: u32,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DisplayMode {
    Web,
    Interactive,
    Stream,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name:&'static str;
        match self {
            Self::Web => name = "web",
            Self::Interactive => name = "interactive",
            Self::Stream => name = "stream"
        };

        write!(f, "{}", name)?;

        Ok(())
    }
}