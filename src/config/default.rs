//! default Action
//!
//! Demonstrates the various logging levels

use crate::config::Config;

/// default Action function
pub fn act(config: &Config) -> i32 {
    println!("Welcome to {}!", config.app);
    error!("example error");
    warn!("example warn");
    info!("example info");
    debug!("example debug");
    trace!("example trace");
    0
}
