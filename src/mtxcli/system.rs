//! Platform specific system information
//!
//! Includes configuration directory

use std::convert::From;
use std::{env,fmt,io};
use std::fs::{File,OpenOptions};
use std::io::prelude::*;
use std::path::{Path,PathBuf};

#[cfg(all(unix, target_os="linux"))]
/// Target operating system name
const TARGET_OS: &str= "linux";
#[cfg(all(unix, target_os="linux"))]
/// Operating system is Linux?
const OS_LINUX: bool = true;
#[cfg(not(all(unix, target_os="linux")))]
/// Operating system is Linux?
const OS_LINUX: bool = false;

#[cfg(windows)]
/// Target operating system name
const TARGET_OS: &str= "windows";
#[cfg(windows)]
/// Operating system is Windows?
const OS_WINDOWS: bool = true;
#[cfg(not(windows))]
/// Operating system is Windows?
const OS_WINDOWS: bool = false;

#[cfg(all(unix, target_os="macos"))]
/// Target operating system name
const TARGET_OS: &str= "macos";
#[cfg(all(unix, target_os="macos"))]
/// Operating system is Mac OS?
const OS_MACOS: bool = true;
#[cfg(not(all(unix, target_os="macos")))]
/// Operating system is Mac OS?
const OS_MACOS: bool = false;

#[cfg(not(any(unix, windows)))]
/// Target operating system name
const TARGET_OS: &str= "UNKNOWN";
#[cfg(not(any(unix, windows)))]
/// Operating system is Unknown?
const OS_UNKNOWN: bool = true;
#[cfg(any(unix, windows))]
/// Operating system is Unknown?
const OS_UNKNOWN: bool = false;

#[cfg(not(windows))]
/// Home directory env var
const HOME: &str= "HOME";
#[cfg(windows)]
/// Home directory env var
const HOME: &str= "USERPROFILE";

/// End of line convention for UNIX
const EOL_UNIX: &str= "\n";
/// End of line convention for Windows
#[allow(dead_code)]
const EOL_WINDOWS: &str= "\r\n";

#[cfg(not(windows))]
/// End of line convention
const EOL: &str= EOL_UNIX;
#[cfg(windows)]
/// End of line convention
const EOL: &str= EOL_WINDOWS;

/// Current working directory
const CWD: &str= ".";

/// The empty string
pub const EMPTY: &str = "";

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
/// Other file ownership mask (octal, UNIX only)
#[allow(dead_code)]
const OTHER_FILE_RW: u32 = 0o177;
#[cfg(unix)]
/// Other directory ownership mask (octal, UNIX only)
#[allow(dead_code)]
const OTHER_DIR_RW: u32 = 0o077;

#[cfg(not(unix))]
/// Sets the file permissions to read and write for the owner (only)
#[allow(dead_code)]
fn owner_rw(_: File) {
}

#[cfg(not(unix))]
/// Sets the path permissions to read and write for the owner (only)
#[allow(dead_code)]
fn path_owner_rw<P: AsRef<Path>>(_path: P) {
}

#[cfg(unix)]
/// Sets the file permissions to read and write for the owner (only)
#[allow(dead_code)]
fn owner_rw(file: File) {
    let metadata = file.metadata()
        .expect("unable to get file permissions");
    let file_type = metadata.file_type();
    let other = if file_type.is_dir() { OTHER_DIR_RW } else { OTHER_FILE_RW };
    let mut perms = metadata.permissions();
    let mut mode = perms.mode();
    if mode & other != 0 {
        mode &= !other;
        perms.set_mode(mode);
        file.set_permissions(perms)
            .expect("unable to set permissions");
    }
}

#[cfg(unix)]
/// Sets the path permissions to read and write for the owner (only)
#[allow(dead_code)]
fn path_owner_rw<P: AsRef<Path>>(path: P) {
    let file = match OpenOptions::new().read(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            error!("could not open path: {:}", e);
            return;
        }
    };
    owner_rw(file);
}

/// Returns the home directory
fn home_dir() -> String  {
    match env::var(HOME) {
        Ok(val) => val,
        Err(e) => {
            error!("couldn't interpret variable name \"{}\": {}", HOME, e);
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
    pub eol: String,
}

/// System implementation
impl System {

    /// Construct a new System
    pub fn new(qualifier: &'static str, organization: &'static str,
               app: &'static str) -> Self {
        System {
            target_os: TARGET_OS,
            config_dir: config_dir(qualifier, organization, app),
            linux: OS_LINUX,
            windows: OS_WINDOWS,
            macos: OS_MACOS,
            unknown: OS_UNKNOWN,
            eol: EOL.to_string()
        }
    }

    /// Returns the value of environment variable `var` or empty string if not set
    pub fn getenv(&self, var: &str) -> String {
        match env::var(var) {
            Ok(val) => val,
            Err(_) => EMPTY.to_string()
        }
    }

    /// converts UNIX line endings to Windows line endtings
    pub fn unix2dos(&self, s: &mut String) {
        *s = s.replace(EOL_UNIX, EOL_WINDOWS);
    }

    /// Reads the file into the string (or default upon failure to read)a
    #[allow(dead_code)]
    pub fn read_file_to_str(&self, mut dst: &mut String, filename: &str, default: &str) {
        dst.clear();
        File::open(filename)
            .and_then(|mut f| f.read_to_string(&mut dst))
            .unwrap_or_else(|_| { dst.push_str(default); dst.len()});
    }

    /// Writes to the file from the string
    #[allow(dead_code)]
    pub fn write_file_from_str(&self, src: &str, filename: &str) -> Result<(), io::Error> {
        let mut psrc = String::from(src);
        if !psrc.is_empty() {
            if let Some(ch) = psrc.get(psrc.len()-1..psrc.len()) {
                if ch != EOL_UNIX {
                    psrc.push_str(EOL_UNIX);
                }
            }
        }
        if self.windows {
            self.unix2dos(&mut psrc);
        }
        let mut path: PathBuf = filename.into();
        if path.pop() {
            std::fs::create_dir_all(&path)
                .expect("unable to make config file directory");
            path_owner_rw(&path);
        }
        let config_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename);
        match config_file {
            Ok(mut f) => f.write_all(psrc.as_bytes()).map(|_| owner_rw(f)),
            Err(e) => Err(e)
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
