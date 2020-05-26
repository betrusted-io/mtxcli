//! Time functions
//!
//!

// use std::convert::TryInto;
// use chrono::{DateTime,NaiveDateTime};
use chrono::DateTime;
use chrono::offset::Utc;
use chrono_tz::{US,Asia};

pub fn now_world() -> String {
    let now: DateTime<Utc> = Utc::now();
    let now_central = now.with_timezone(&US::Central);
    let now_sgt = now.with_timezone(&Asia::Singapore);
    let mut world = String::new();
    world.push('🕰');
    world.push(' ');
    world.push(' ');
    world.push_str(&now_central.format("%Z %m/%d %H:%M").to_string());
    world.push_str(&now.format(" | %Z %m/%d/%Y %T.%f").to_string());
    world.push_str(&now_sgt.format(" | SGT %m/%d %H:%M").to_string());
    return world
}
