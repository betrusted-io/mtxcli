//! ParkingLot Action
//!
//! Function of reminders and for using (as yet) unused symbols

use crate::mtxcli;

/// ParkingLot Action function
pub fn act(mtxcli: &mtxcli::Mtxcli) -> i32  {
    println!("parking lot");
    if mtxcli.args.verbose > 0 {
        println!("USER = {}", mtxcli.system.getenv("USER"));
    }
    0
}
