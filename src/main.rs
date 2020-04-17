extern crate clap;
extern crate directories;

use clap::{App, Arg};
use directories::ProjectDirs;

// example of conditional compilation


#[cfg(all(unix, target_os="linux"))]
const TARGET_OS: &'static str= "linux";
#[cfg(all(unix, target_os="linux"))]
const OS_LINUX: bool = true;
#[cfg(not(all(unix, target_os="linux")))]
const OS_LINUX: bool = false;

#[cfg(windows)]
const TARGET_OS: &'static str= "windows";
#[cfg(windows)]
const OS_WINDOWS: bool = true;
#[cfg(not(windows))]
const OS_WINDOWS: bool = false;

#[cfg(all(unix, target_os="macos"))]
const TARGET_OS: &'static str= "macos";
#[cfg(all(unix, target_os="macos"))]
const OS_MACOS: bool = true;
#[cfg(not(all(unix, target_os="macos")))]
const OS_MACOS: bool = false;

#[cfg(not(any(unix, windows)))]
const TARGET_OS: &'static str= "UNKNOWN";
#[cfg(not(any(unix, windows)))]
const OS_UNKNOWN: bool = true;
#[cfg(any(unix, windows))]
const OS_UNKNOWN: bool = false;

/// mtxcli version
const VERSION: &'static str= "0.2.0";

/// The main **mtxcli** program
fn main() {
    let matches = App::new("mtxcli")
        .version(VERSION)
        .about("Matrix Command Line Interface")
        .arg(Arg::with_name("assign")
             .short("a")
             .long("assign")
             .value_name("var")
             .help("sets var = value (not saved)")
             .takes_value(true)
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets a custom config file")
             .takes_value(true))
        .arg(Arg::with_name("show-config")
             .short("C")
             .long("show-config")
             .help("Show config values"))
        .arg(Arg::with_name("get")
             .short("g")
             .long("get")
             .value_name("var")
             .help("gets var's value")
             .takes_value(true)
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("set")
             .short("s")
             .long("set")
             .value_name("var=value")
             .help("sets var = value (saved)")
             .takes_value(true)
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("config-path")
             .short("P")
             .long("config-path")
             .help("Show config file path"))
        .get_matches();
    let proj_dirs = ProjectDirs::from("io", "Betrusted", "mtxcli")
        .expect("could not find default config directory");
    let config_default = proj_dirs.config_dir().to_str()
        .expect("could not convert config_dir to a string");
    let config_dir = matches.value_of("config")
        .unwrap_or_else(|| config_default);

    if matches.is_present("config-path") {
        println!("config: {}", config_dir);
    }
    if let Some(sets) = matches.values_of("set") {
        for set in sets {
            println!("set: {}", set);
        }
    }
    if let Some(assigns) = matches.values_of("assign") {
        for assign in assigns {
            println!("assign: {}", assign);
        }
    }
    if let Some(gets) = matches.values_of("get") {
        for get in gets {
            println!("get: {}", get);
        }
    }

    println!("Welcome to mtxcli!");
    println!("TARGET_OS.: {}", TARGET_OS);
    println!("OS_LINUX..: {}", OS_LINUX);
    println!("OS_WINDOWS: {}", OS_WINDOWS);
    println!("OS_MACOS..: {}", OS_MACOS);
    println!("OS_UNKNOWN: {}", OS_UNKNOWN);
}
