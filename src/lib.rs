/// Support for wownow
#[warn(clippy::pedantic)]
#[warn(missing_docs)]
#[warn(clippy::cargo)]
#[allow(clippy::multiple_crate_versions)]

pub(crate) mod response {
    pub(crate) mod base;
    pub(crate) mod summary;
    pub(crate) mod versions;
}
pub(crate) mod api;
pub(crate) mod output;

pub mod prelude {
    pub use crate::{
        api::{get_summary, get_versions, Error as ApiError, Result as ApiResult},
        output::{Error as OutputError, Product, Result as OutputResult, Version, VersionsFetch},
        response::{
            base::{Dec4, Error as ResponseError, Hex16, String0},
            summary::{Record as SummaryRecord, Response as SummaryResponse},
            versions::{Record as VersionsRecord, Response as VersionsResponse},
        },
    };
}
