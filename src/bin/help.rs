use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Gibrid", about = "Ground state search software for spin systems")]
struct Args {
    /// output mfsys file with spin system
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    /// max thread count
    threads: usize,
    /// input mfsys file with spin system
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
    /// lattice type and parameters
    lattice: Option<String>,
}

fn main() {
    let opt = Args::from_args();
}