pub mod catalog_file;
mod errors;
mod geojson_utils;
mod hash_utils;
pub mod input_file;
mod s2_utils;

use catalog_file::CatalogFile;
use input_file::InputFile;
use std::path::PathBuf;

use crate::errors::Result;

pub async fn update_catalog(file: PathBuf, api_prefix: PathBuf) -> Result<()> {
    // Read the input file
    let file = InputFile::new(file).await?;
    // copy it to the API
    file.copy_to_api(api_prefix.clone()).await?;

    for cell in &file.s2_cells {
        let catalog_filename = format!("{}.csv", cell.to_token());
        let catalog_path = api_prefix.join("catalog").join(catalog_filename);
        println!("Catalog path: {:?}", catalog_path);
        let mut catalog = CatalogFile::new(catalog_path);
        catalog.read()?;
        catalog.add_file(&file)?;
        catalog.write()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_update_catalog() {}
}
