//! ParkingLot Action
//!
//! Function of reminders and for using (as yet) unused symbols

use crate::config;
use serde_json::json;

/// ParkingLot Action function
pub fn act(config: &config::Config) -> i32  {
    println!("parking lot");
    println!("USER = {}", config.system.getenv("USER"));
    config::print_json("dummy", &json!("dummy"));
    0
}
