//! ParkingLot Action
//!
//! Function of reminders and for using (as yet) unused symbols

use crate::config::Config;

/// ParkingLot Action function
pub fn act(config: &Config) -> i32  {
    println!("parking lot");
    println!("USER = {}", config.system.getenv("USER"));
    0
}
