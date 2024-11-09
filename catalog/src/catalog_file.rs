use crate::{
    errors::{Error, Result},
    input_file::InputFile,
    s2_utils,
};
use csv::{ReaderBuilder, WriterBuilder};
use s2::cellid::CellID;
use std::{fs, path::PathBuf};

#[derive(Debug)]
enum MetadataValue {
    Version(u8),
    Count(u64),
}

#[derive(Debug, Clone)]
pub struct SingleFile {
    kind: u8,
    display_name: String,
    hash: String,
    coverage: f64,
}

impl SingleFile {
    fn new_within_catalog(catalog: &ParsedCatalogFile, file: &InputFile) -> Self {
        let polygon = &file.outline;
        let coverage = s2_utils::coverage_of_cell(polygon, &catalog.cell_id);
        let hash = file.hash.clone();
        SingleFile {
            kind: 1,
            display_name: file.display_name.clone(),
            hash,
            coverage,
        }
    }
}

#[derive(Debug)]
struct ParsedCatalogFile {
    cell_id: CellID,
    metadata: Vec<MetadataValue>,
    files: Vec<SingleFile>,
}

impl ParsedCatalogFile {
    fn new(cell_id: CellID) -> Self {
        ParsedCatalogFile {
            cell_id,
            metadata: vec![MetadataValue::Version(1), MetadataValue::Count(0)],
            files: Vec::new(),
        }
    }

    fn update_metadata(&mut self, updater: fn(&mut MetadataValue) -> ()) {
        for value in &mut self.metadata {
            updater(value);
        }
    }
}

pub struct CatalogFile {
    file_path: PathBuf,
    cell_id: CellID,
    parsed_file: Option<ParsedCatalogFile>,
}

impl CatalogFile {
    pub fn new(file: PathBuf) -> Self {
        let cell_token = file
            .file_stem()
            .expect("No file stem found")
            .to_str()
            .expect("Could not convert file stem to string");
        let cell_id = CellID::from_token(cell_token);

        CatalogFile {
            file_path: file,
            parsed_file: None,
            cell_id,
        }
    }

    fn parse_file(&mut self) -> Result<()> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true) // this is required to allow for variable number of columns
            .from_path(self.file_path.clone())?;

        let mut metadata = Vec::new();
        let mut files = Vec::new();

        for line in rdr.records() {
            let record = line?;
            match record.get(0) {
                Some("meta") => match record.get(1) {
                    Some("version") => {
                        let version = record.get(2).ok_or("No version found")?;
                        metadata.push(MetadataValue::Version(version.parse()?));
                    }
                    Some("count") => {
                        let count = record.get(2).ok_or("No count found")?;
                        metadata.push(MetadataValue::Count(count.parse()?));
                    }
                    _ => (),
                },
                Some("file") => {
                    let kind = record.get(1).ok_or("No kind found")?;
                    let display_name = record.get(2).ok_or("No display name found")?;
                    let hash = record.get(3).ok_or("No hash found")?;
                    let coverage = record.get(4).ok_or("No coverage found")?;
                    files.push(SingleFile {
                        kind: kind.parse()?,
                        display_name: display_name.to_string(),
                        hash: hash.to_string(),
                        coverage: coverage.parse()?,
                    });
                }
                _ => (),
            }
        }

        self.parsed_file = Some(ParsedCatalogFile {
            cell_id: self.cell_id,
            metadata,
            files,
        });
        Ok(())
    }

    /**
     * Reads the catalog file and parses its contents. If the file does not exist, an empty ParsedCatalogFile is created.
     */
    pub fn read(&mut self) -> Result<()> {
        match self.parse_file() {
            Ok(_) => Ok(()),
            Err(Error::CSVError(_)) => {
                self.parsed_file = Some(ParsedCatalogFile::new(self.cell_id));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn write_to(&mut self, path: PathBuf) -> Result<()> {
        let mut wtr = WriterBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_path(path)?;

        let parsed_file = self
            .parsed_file
            .as_ref()
            .ok_or(Error::CatalogFileNotParsed)?;

        for value in &parsed_file.metadata {
            let record: Vec<String> = match value {
                MetadataValue::Version(version) => {
                    vec![
                        "meta".to_string(),
                        "version".to_string(),
                        version.to_string(),
                    ]
                }
                MetadataValue::Count(count) => {
                    vec!["meta".to_string(), "count".to_string(), count.to_string()]
                }
            };
            wtr.write_record(record)?;
        }

        for file in &parsed_file.files {
            wtr.write_record(vec![
                "file".to_string(),
                file.kind.to_string(),
                file.display_name.clone(),
                file.hash.clone(),
                file.coverage.to_string(),
            ])?;
        }

        wtr.flush()?;

        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        let file_path = self.file_path.clone();
        let dirname = file_path
            .parent()
            .ok_or(Error::GenericError("No parent directory".to_string()))?;
        fs::create_dir_all(dirname)?;
        self.write_to(file_path)
    }

    pub fn add_file(&mut self, file: &InputFile) -> Result<()> {
        let parsed_file = self
            .parsed_file
            .as_mut()
            .ok_or(Error::CatalogFileNotParsed)?;

        let single_file = SingleFile::new_within_catalog(parsed_file, file);

        // check if the file is already in the catalog
        // if it is, we'll replace it with the new one
        // if it isn't well update the count and add it to the list
        let existing_file_idx = parsed_file.files.iter().position(|f| f.hash == file.hash);
        if let Some(pos) = existing_file_idx {
            parsed_file.files[pos] = single_file;
            return Ok(());
        } else {
            parsed_file.update_metadata(|value| match value {
                MetadataValue::Count(count) => *count += 1,
                _ => (),
            });
            parsed_file.files.push(single_file);
        }

        parsed_file
            .files
            .sort_by(|a, b| b.coverage.partial_cmp(&a.coverage).unwrap());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_roundtrip() -> Result<()> {
        let mut catalog_file = CatalogFile::new(PathBuf::from("./examples/basic-catalog.csv"));
        catalog_file.read()?;
        // println!("{:?}", catalog_file.parsed_file);
        catalog_file.write_to(PathBuf::from("./examples/basic-catalog-roundtrip.csv"))?;

        // Read both files as strings
        let original_content = fs::read_to_string("./examples/basic-catalog.csv")?;
        let roundtrip_content = fs::read_to_string("./examples/basic-catalog-roundtrip.csv")?;

        // Assert that the contents are the same
        assert_eq!(
            original_content, roundtrip_content,
            "The contents of the files do not match!"
        );

        Ok(())
    }
}
