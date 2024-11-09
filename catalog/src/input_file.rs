use crate::{errors::Result, geojson_utils, hash_utils::sha256_file, s2_utils};

use geo::MultiPolygon;
use pmtiles::async_reader::AsyncPmTilesReader;
use s2::cellid::CellID;
use std::path::PathBuf;

async fn get_outline(path: &PathBuf) -> Result<MultiPolygon<f64>> {
    let geojson_path = path.with_extension("geojson");
    let geojson = tokio::fs::read_to_string(geojson_path).await?;
    let polygon = geojson_utils::geojson_to_union(geojson)?;
    Ok(polygon)
}

async fn get_name_from_reader(
    reader: &AsyncPmTilesReader<pmtiles::MmapBackend>,
    filename: String,
) -> Result<String> {
    let metadata = reader.get_metadata().await?;
    let v: serde_json::Value = serde_json::from_str(&metadata)?;
    let display_name = &v["name"];
    Ok(display_name.as_str().unwrap_or(&filename).to_string())
}

pub struct InputFile {
    pub path: PathBuf,
    pub s2_cells: Vec<CellID>,
    pub outline: MultiPolygon<f64>,
    pub display_name: String,
    pub hash: String,
    // pmtiles_reader: AsyncPmTilesReader<pmtiles::MmapBackend>,
}
impl InputFile {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let outline = get_outline(&path).await?;
        let s2_cells = s2_utils::covering_s2_cells(&outline);
        println!("S2 cells: {:?}", s2_cells);
        let pmtiles_reader = AsyncPmTilesReader::new_with_path(&path).await?;
        let display_name = get_name_from_reader(
            &pmtiles_reader,
            path.file_name().unwrap().to_str().unwrap().to_string(),
        )
        .await?;
        let hash = sha256_file(path.clone()).await?;
        Ok(InputFile {
            path,
            s2_cells,
            outline,
            // pmtiles_reader,
            display_name,
            hash,
        })
    }

    pub async fn copy_to_api(&self, api_prefix: PathBuf) -> Result<()> {
        let files_path = api_prefix.join("files");
        tokio::fs::create_dir_all(&files_path).await?;

        let path_stem = files_path.join(&self.hash);
        let input_path_ext = self.path.extension().unwrap();

        tokio::fs::copy(&self.path, path_stem.with_extension(input_path_ext)).await?;
        tokio::fs::copy(
            self.path.with_extension("geojson"),
            path_stem.with_extension("geojson"),
        )
        .await?;

        Ok(())
    }
}
