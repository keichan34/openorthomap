use catalog::catalog_file::SingleFile;
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
        let geojson_path = input.with_extension("geojson");
        if !geojson_path.exists() {
            println!(
                "GeoJSON file not found for PMTiles file: {:?}, skipping",
                input
            );
            continue;
        }
        println!("GeoJSON file found: {:?}", geojson_path);

        let file = SingleFile::new(1, input.to_string_lossy().to_string());
        let geojson = std::fs::read_to_string(geojson_path).unwrap();
        match catalog::update_catalog(file, geojson, api_dir.clone()) {
            Ok(_) => println!("Catalog updated successfully"),
            Err(e) => eprintln!("Error updating catalog: {:?}", e),
        }
    }
}
