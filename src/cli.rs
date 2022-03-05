use clap::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    /// The localhost port number to use.
    #[structopt(long, default_value = "4444")]
    pub port: u16,
}
