use crate::errors::{Error, Result};
use csv::{ReaderBuilder, WriterBuilder};
use std::{fs, path::PathBuf};

#[derive(Debug)]
enum MetadataValue {
    Version(u8),
    Count(u64),
    SubcellCount(u8, u64),
}

#[derive(Debug, Clone)]
pub struct SingleFile {
    kind: u8,
    filename: String,
}

impl SingleFile {
    pub fn new(kind: u8, filename: String) -> Self {
        SingleFile { kind, filename }
    }
}

#[derive(Debug)]
struct ParsedCatalogFile {
    metadata: Vec<MetadataValue>,
    files: Vec<SingleFile>,
}

impl ParsedCatalogFile {
    fn new() -> Self {
        ParsedCatalogFile {
            metadata: vec![
                MetadataValue::Version(1),
                MetadataValue::Count(0),
                // MetadataValue::SubcellCount(0, 0),
                // MetadataValue::SubcellCount(1, 0),
                // MetadataValue::SubcellCount(2, 0),
                // MetadataValue::SubcellCount(3, 0),
            ],
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
    parsed_file: Option<ParsedCatalogFile>,
}

impl CatalogFile {
    pub fn new(file: PathBuf) -> Self {
        CatalogFile {
            file_path: file,
            parsed_file: None,
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
                    Some("subcell-count") => {
                        let sub_index = record.get(2).ok_or("No subcell index found")?;
                        let count = record.get(3).ok_or("No count found")?;
                        metadata.push(MetadataValue::SubcellCount(
                            sub_index.parse()?,
                            count.parse()?,
                        ));
                    }
                    _ => (),
                },
                Some("file") => {
                    let kind = record.get(1).ok_or("No kind found")?;
                    let filename = record.get(2).ok_or("No filename found")?;
                    files.push(SingleFile {
                        kind: kind.parse()?,
                        filename: filename.to_string(),
                    });
                }
                _ => (),
            }
        }

        self.parsed_file = Some(ParsedCatalogFile { metadata, files });
        Ok(())
    }

    /**
     * Reads the catalog file and parses its contents. If the file does not exist, an empty ParsedCatalogFile is created.
     */
    pub fn read(&mut self) -> Result<()> {
        match self.parse_file() {
            Ok(_) => Ok(()),
            Err(Error::CSVError(_)) => {
                self.parsed_file = Some(ParsedCatalogFile::new());
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
                MetadataValue::SubcellCount(sub_index, count) => vec![
                    "meta".to_string(),
                    "subcell-count".to_string(),
                    sub_index.to_string(),
                    count.to_string(),
                ],
            };
            wtr.write_record(record)?;
        }

        for file in &parsed_file.files {
            wtr.write_record(vec![
                "file".to_string(),
                file.kind.to_string(),
                file.filename.clone(),
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

    pub fn add_file(&mut self, file: SingleFile) -> Result<()> {
        let parsed_file = self
            .parsed_file
            .as_mut()
            .ok_or(Error::CatalogFileNotParsed)?;
        if !parsed_file
            .files
            .iter()
            .any(|f| f.filename == file.filename)
        {
            parsed_file.files.push(file);
            parsed_file.update_metadata(|value| match value {
                MetadataValue::Count(count) => *count += 1,
                _ => (),
            });
        }
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
