//! **mtxcli**
//!
//! Matrix Command Line Interface

#[macro_use] extern crate log;
use std::process;
mod config;
use serde_json::Map;

/// qualifier
const QUALIFIER: &str = "io";
/// organization
const ORGANIZATION: &str = "Betrusted";
/// application
const APP: &str = "mtxcli";
/// version
const VERSION: &str= "0.4.0";

/// The main **mtxcli** program
fn main() {
    let mut map = Map::new();
    let mut config = config::Config::new(QUALIFIER, ORGANIZATION, APP, VERSION, &mut map);
    config.parse_args();
    process::exit(config.act());
}
