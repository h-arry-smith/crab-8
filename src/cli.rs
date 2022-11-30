use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the chip-8 rom that you want to run
    pub path: String,

    /// Display debug output when running a chip-8 rom
    #[arg(short, long)]
    pub debug: bool,
}
