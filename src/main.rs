//! **mtxcli**
//!
//! Matrix Command Line Interface

#[macro_use] extern crate log;
use std::process;
mod config;

/// qualifier
const QUALIFIER: &str = "io";
/// organization
const ORGANIZATION: &str = "Betrusted";
/// application
const APP: &str = "mtxcli";
/// version
const VERSION: &str= "0.2.1";

/// The main **mtxcli** program
fn main() {
    let mut config = config::Config::new(QUALIFIER, ORGANIZATION, APP, VERSION);
    process::exit(config.act());
}
