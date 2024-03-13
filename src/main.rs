//! A CLI tool to get the current versions of World of Warcraft
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use clap::Parser;
use serde_json::{to_string, to_string_pretty};
use std::process::ExitCode;
use tokio::task::JoinSet;
use wownow::prelude::*;

struct RunConfig {
    live_only: bool,
    pretty_print: bool,
}

fn resolve_switched_arg(yes: bool, no: bool, default: bool) -> bool {
    match (yes, no) {
        (true, false) => true,
        (false, true) => false,
        (false, false) => default,
        (true, true) => unreachable!("clap should prevent this"),
    }
}

impl From<Args> for RunConfig {
    fn from(args: Args) -> Self {
        RunConfig {
            live_only: resolve_switched_arg(args.live_only, args.no_live_only, true),
            pretty_print: resolve_switched_arg(args.pretty, args.no_pretty, true),
        }
    }
}

// eeek, there's no nice way to make clap `--no-*` switches. we follow this advice:
// https://github.com/clap-rs/clap/discussions/5177
/// Get the current versions of World of Warcraft
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// Only return products that are traditionally "live", or playable by most users. This is the
    /// products named `wow`, `wow_classic`, and `wow_classic_era`.
    ///
    /// Defaults to on. Turn off with `--no-live-only`.
    #[arg(long, overrides_with("no_live_only"))]
    live_only: bool,
    #[arg(long, overrides_with("live_only"), hide(true))]
    no_live_only: bool,

    /// Pretty print the JSON output.
    ///
    /// Defaults to on. Turn off with `--no-pretty`.
    #[arg(long, overrides_with("no_pretty"))]
    pretty: bool,
    #[arg(long, overrides_with("pretty"), hide(true))]
    no_pretty: bool,
}

type Result = std::result::Result<String, String>;

const LIVE_PRODUCTS: [&str; 3] = ["wow", "wow_classic", "wow_classic_era"];

async fn run(config: RunConfig) -> Result {
    let summary = get_summary()
        .await
        .map_err(|e| format!("Error getting summary: {e}"))?;

    let matching_products = summary
        .records
        .into_iter()
        .filter_map(|record| {
            // Only return products are that live (if called for by user) and have no flags. flags
            // indicate things like cdn or bgdl which we don't care about, we just want the normal
            // one.
            if (!config.live_only || LIVE_PRODUCTS.contains(&record.product.as_str()))
                && record.flags.is_empty()
            {
                Some(record.product)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut set = JoinSet::new();
    for matching_product in matching_products {
        set.spawn(async move {
            get_versions(&matching_product)
                .await
                .map(|resp| (resp, matching_product.clone()))
                .map_err(|e| (e, matching_product))
        });
    }

    let mut fetch = VersionsFetch::new();
    while let Some(join_result) = set.join_next().await {
        let response_result = join_result.map_err(|e| format!("Error joining task: {e}"))?;
        let (response, product_name) = response_result.map_err(|(error, product_name)| {
            format!("Error getting `{product_name}` versions: {error}")
        })?;

        fetch.add_product(Product::from_versions_response(&product_name, &response));
    }

    let output = if config.pretty_print {
        to_string_pretty(&fetch).map_err(|e| format!("Error serializing JSON: {e}"))?
    } else {
        to_string(&fetch).map_err(|e| format!("Error serializing JSON: {e}"))?
    };

    Ok(output)
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    match run(args.into()).await {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}
