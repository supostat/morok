#![forbid(unsafe_code)]

use std::process::ExitCode;

use morok_catalog::{Catalog, Page};

fn main() -> ExitCode {
    match Catalog::launch(Page::Buttons) {
        Ok(catalog) => {
            catalog.run();
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("morok-catalog: {error}");
            ExitCode::FAILURE
        }
    }
}
