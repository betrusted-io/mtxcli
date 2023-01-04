//! **mtxcli**
//!
//! Matrix Command Line Interface

#[macro_use] extern crate log;
use std::process;
mod mtxcli;

/// qualifier
const QUALIFIER: &str = "io";
/// organization
const ORGANIZATION: &str = "Betrusted";
/// application
const APP: &str = env!("CARGO_PKG_NAME");
/// version
const VERSION: &str= env!("CARGO_PKG_VERSION");

/// The main **mtxcli** program
fn main() {
    let mut mtxcli = mtxcli::Mtxcli::new(QUALIFIER, ORGANIZATION, APP, VERSION);
    mtxcli.parse_args();
    let rc = mtxcli.act();
    process::exit(rc);
}
