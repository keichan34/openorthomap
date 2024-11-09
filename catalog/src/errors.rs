use thiserror::Error;

/// Errors that can occur in this library.
#[derive(Error, Debug)]
pub enum Error {
    #[error("An error occurred while parsing GeoJSON: `{0}`")]
    GeoJSONError(#[from] geojson::Error),

    #[error("An error occurred while reading CSV: `{0}`")]
    CSVError(#[from] csv::Error),

    #[error("An IO error occurred: `{0}`")]
    IOError(#[from] std::io::Error),

    #[error("An error occurred while parsing an integer: `{0}`")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("An error occurred while parsing a float: `{0}`")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("An error occurred while accessing the PMTiles archive: `{0}`")]
    PMTilesError(#[from] pmtiles::PmtError),

    #[error("An error occurred while parsing JSON: `{0}`")]
    JSONError(#[from] serde_json::Error),

    #[error("CatalogFile must be parsed first before accessing its contents")]
    CatalogFileNotParsed,

    #[error("A generic error occurred: `{0}`")]
    GenericError(String),
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::GenericError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
