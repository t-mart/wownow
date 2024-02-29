//! A stupid CLI tool to get the current World of Warcraft versions
//!
//! The nice folks at <https://wago.tools/> show a table with the current versions of the game. This
//! tool scrapes that table and returns a JSON with the versions.
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::cargo)]

use std::process::ExitCode;

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::{error::SelectorErrorKind, Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
enum ParseError {
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Version unparseable into (version, build)")]
    VersionStrUnparseable,
}

#[derive(ThisError, Debug)]
enum WagoGetError {
    /// Some reqwest error during the GET request or reading the body
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Selector should be parseable
    #[error("Selector not parseable")]
    ScraperSelectorErrorKind(#[from] SelectorErrorKind<'static>),

    /// Element with the id `app` not found
    #[error("No `div#app` found")]
    NoAppDivFound,

    /// No `data-page` attribute found in the `div#app` element
    #[error("No `data-page` attr found")]
    NoDataPageAttrFound,

    /// Couldn't parse the JSON in the `data-page` attribute
    #[error(transparent)]
    JsonParse(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct WagoDataPage {
    props: WagoDataPageProps,
}

#[derive(Debug, Deserialize)]
struct WagoDataPageProps {
    versions: Vec<WagoDataPageVersion>,
}

#[derive(Debug, Deserialize)]
struct WagoDataPageVersion {
    /// Ex: wow_classic_era
    product: String,

    /// Ex: 1.15.1.53495
    version: String,

    /// Ex: 2023-11-22 18:06:03
    created_at: String,
}

impl WagoDataPage {
    /// Fetches the data page from <https://wago.tools/> and returns a `WagoDataPage` instance.
    ///
    /// # Errors
    ///
    /// This method can return an [`Err`] of with a [`WagoGetError`]. See its documentation for more
    /// information.
    fn http_get() -> Result<Self, WagoGetError> {
        let url = "https://wago.tools/";
        let body = reqwest::blocking::get(url)?.text()?;

        // does this panic? seems like it should if its not returning a result
        let doc = Html::parse_document(&body);
        let app_div = Selector::parse("div#app")?;

        Ok(serde_json::from_str(
            doc.select(&app_div)
                .next()
                .ok_or(WagoGetError::NoAppDivFound)?
                .attr("data-page")
                .ok_or(WagoGetError::NoDataPageAttrFound)?,
        )?)
    }
}

/// Our display format struct. Almost same as [`WagoDataPageVersion`], but massaged to our needs.
#[derive(Debug, Serialize)]
struct Version {
    /// Ex: wow_classic_era
    product: String,

    /// Ex: 1.15.1
    #[allow(clippy::struct_field_names)]
    version: String,

    /// Ex: 53495
    build: String,

    /// Ex: 2023-11-22T18:06:03Z
    created_at: DateTime<Utc>,
}

impl TryFrom<WagoDataPageVersion> for Version {
    type Error = ParseError;

    fn try_from(wago_version: WagoDataPageVersion) -> Result<Self, Self::Error> {
        let Some((version, build_str)) = wago_version.version.rsplit_once('.') else {
            return Err(ParseError::VersionStrUnparseable);
        };
        let created_at =
            NaiveDateTime::parse_from_str(&wago_version.created_at, "%Y-%m-%d %H:%M:%S")?.and_utc();

        Ok(Self {
            product: wago_version.product,
            version: version.to_owned(),
            build: build_str.to_owned(),
            created_at,
        })
    }
}

impl TryFrom<WagoDataPage> for Vec<Version> {
    type Error = ParseError;

    fn try_from(wago_data_page: WagoDataPage) -> Result<Self, Self::Error> {
        wago_data_page
            .props
            .versions
            .into_iter()
            .map(Version::try_from)
            .collect()
    }
}

fn main() -> ExitCode {
    let data_page_res = WagoDataPage::http_get();

    let data_page = match data_page_res {
        Ok(data_page) => data_page,
        Err(err) => {
            eprintln!("Error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let versions_res = Vec::<Version>::try_from(data_page);

    let versions = match versions_res {
        Ok(versions) => versions,
        Err(err) => {
            eprintln!("Error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let json_string = serde_json::to_string_pretty(&versions);

    match json_string {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
