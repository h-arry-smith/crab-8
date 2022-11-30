use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the chip-8 rom that you want to run
    pub path: String,

    /// Display debug output when running a chip-8 rom
    #[arg(short, long)]
    pub debug: bool,

    /// Set the color in hex (e.g #FF0000) for pixels that are on
    #[arg(short, long)]
    pub fg: Option<String>,

    /// Set the color in hex (e.g #00FF00) for pixels that are off
    #[arg(short, long)]
    pub bg: Option<String>,

    /// Start the emulator in ETI 660 Mode
    #[arg(short, long)]
    pub eti_mode: bool,
}
