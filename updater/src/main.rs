use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Path to the root API directory (the directory /api/v1/ is located in). Defaults to the current directory."
    )]
    root_dir: Option<PathBuf>,

    #[arg(
        help = "Paths of files to be added to the API. Currently, only PMTiles with accopmanying GeoJSON extents are supported.",
        required = true
    )]
    input: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
