pub mod catalog_file;
mod errors;
mod geojson_utils;
mod s2_utils;

use std::path::PathBuf;

use catalog_file::{CatalogFile, SingleFile};
use s2::cellid::CellID;

use crate::errors::Result;

fn geojson_to_s2_cells(geojson: String) -> Result<Vec<CellID>> {
    let union = geojson_utils::geojson_to_union(geojson)?;
    Ok(s2_utils::covering_s2_cells(&union))
}

pub fn update_catalog(
    file: SingleFile,
    file_outline_geojson: String,
    api_prefix: PathBuf,
) -> Result<()> {
    let s2_cells = geojson_to_s2_cells(file_outline_geojson)?;

    for cell in s2_cells {
        let catalog_filename = format!("{}.csv", cell.to_token());
        let catalog_path = api_prefix.join(catalog_filename);
        println!("Catalog path: {:?}", catalog_path);
        let mut catalog = CatalogFile::new(catalog_path);
        catalog.read()?;
        catalog.add_file(file.clone())?;
        catalog.write()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_update_catalog() {}
}
