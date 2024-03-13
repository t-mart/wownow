use crate::response::versions::{Record as VersionsRecord, Response as VersionsResponse};
use serde::Serialize;

/// Errors that can occur when parsing a response.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unparseable version field: {0}")]
    UnparseableVersion(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub struct VersionsFetch {
    retrieval_datetime: chrono::DateTime<chrono::Utc>,
    products: Vec<Product>,
}

impl VersionsFetch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_product(&mut self, product: Product) {
        self.products.push(product);
    }
}

impl Default for VersionsFetch {
    fn default() -> Self {
        Self {
            retrieval_datetime: chrono::Utc::now(),
            products: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Product {
    name: String,
    versions: Vec<Version>,
}

impl Product {
    pub fn from_versions_response(name: &str, response: &VersionsResponse) -> Self {
        let versions = response
            .records
            .iter()
            .map(Version::try_from)
            .collect::<Result<_>>()
            .unwrap();
        Self {
            name: name.to_owned(),
            versions,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Version {
    region: String,
    version: String,
    build: String,
}

impl TryFrom<&VersionsRecord> for Version {
    type Error = Error;

    fn try_from(record: &VersionsRecord) -> Result<Self> {
        let region = record.region.clone();
        let Some((version, build)) = record.versions_name.rsplit_once('.') else {
            return Err(Error::UnparseableVersion(record.versions_name.clone()));
        };
        Ok(Version {
            region,
            version: version.to_owned(),
            build: build.to_owned(),
        })
    }
}
