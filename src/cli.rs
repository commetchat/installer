use clap::{arg, command, Parser};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "install")]
    pub command: String,

    #[arg(short, long)]
    pub build: Option<String>,

    #[arg(short, long, default_value_t = false)]
    pub prerelease: bool,
}
