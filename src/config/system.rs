//! Platform specific system information
//!
//! Includes configuration directory

use std::env;
use std::fmt;
use std::path::PathBuf;

#[cfg(all(unix, target_os="linux"))]
const TARGET_OS: &str= "linux";
#[cfg(all(unix, target_os="linux"))]
const OS_LINUX: bool = true;
#[cfg(not(all(unix, target_os="linux")))]
const OS_LINUX: bool = false;

#[cfg(windows)]
const TARGET_OS: &str= "windows";
#[cfg(windows)]
const OS_WINDOWS: bool = true;
#[cfg(not(windows))]
const OS_WINDOWS: bool = false;

#[cfg(all(unix, target_os="macos"))]
const TARGET_OS: &str= "macos";
#[cfg(all(unix, target_os="macos"))]
const OS_MACOS: bool = true;
#[cfg(not(all(unix, target_os="macos")))]
const OS_MACOS: bool = false;

#[cfg(not(any(unix, windows)))]
const TARGET_OS: &str= "UNKNOWN";
#[cfg(not(any(unix, windows)))]
const OS_UNKNOWN: bool = true;
#[cfg(any(unix, windows))]
const OS_UNKNOWN: bool = false;

/// Home directory
#[cfg(not(windows))]
const HOME: &str= "HOME";
#[cfg(windows)]
const HOME: &str= "USERPROFILE";

/// current working directory
const CWD: &str= ".";

/// Returns the home directory
fn home_dir() -> String  {
    match env::var(HOME) {
        Ok(val) => val,
        Err(e) => {
            println!("couldn't interpret variable name \"{}\": {}", HOME, e);
            CWD.into()
        }
    }
}

/// Returns the platform specific configuration directory for this application
fn config_dir(qualifier: &'static str, organization: &'static str,
              app: &'static str) -> String {
    let mut path: PathBuf = home_dir().into();
    if OS_LINUX {
        path.push(".config");
        path.push(app);
    } else if OS_WINDOWS {
        path.push("AppData");
        path.push("Roaming");
        path.push(organization);
        path.push(app);
    } else if OS_MACOS {
        path.push("Library");
        path.push("Preferences");
        let mut qoa = String::from(qualifier);
        qoa.push('.');
        qoa.push_str(organization);
        qoa.push('.');
        qoa.push_str(app);
        path.push(qoa);
    }
    match path.to_str() {
        Some(path_str) => path_str.to_string(),
        None => home_dir()
    }
}

/// System struct
#[derive(Debug, PartialEq)]
pub struct System {
    pub target_os: &'static str,
    pub config_dir: String,
    pub linux: bool,
    pub windows: bool,
    pub macos: bool,
    pub unknown: bool,
}

/// System implementation
impl System {
    pub fn new(qualifier: &'static str, organization: &'static str,
               app: &'static str) -> Self {
        System {
            target_os: TARGET_OS,
            config_dir: config_dir(qualifier, organization, app),
            linux: OS_LINUX,
            windows: OS_WINDOWS,
            macos: OS_MACOS,
            unknown: OS_UNKNOWN,
        }
    }
}

/// simple Display for System
impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "target_os: {}, config_dir: {}",
               self.target_os, self.config_dir)
    }
}
