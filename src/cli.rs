/// Implementation of the cli for the tool
use clap::{Parser, Subcommand, ValueEnum};

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
    Adsb {
        #[arg(short, long)]
        device: Option<u32>,

        #[arg(short = 'm', long = "mode", default_value_t = DisplayMode::Stream)]
        mode: DisplayMode,
    }
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