//! ShowConfig Action
//!
//! Prints basic configuration information

use crate::config::Config;

/// ShowConfig Action function
pub fn act(config: &Config) -> i32  {
    println!("config: {}", config);
    0
}
