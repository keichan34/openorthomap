use catalog;
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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let root_dir = args.root_dir.unwrap_or_else(|| PathBuf::from("."));
    let api_dir = root_dir.join("api/v1");
    println!("API output directory: {:?}", api_dir);
    for input in args.input {
        println!("Adding file: {:?}", input);
        let extname = input.extension().unwrap();
        if extname != "pmtiles" {
            println!("Unsupported file type: {:?}, skipping", extname);
            continue;
        }
        let res = catalog::update_catalog(input.clone(), api_dir.clone()).await;
        match res {
            Ok(_) => println!("Added {} to catalog", input.display()),
            Err(e) => eprintln!("Error updating catalog: {:?}", e),
        }
    }
}
