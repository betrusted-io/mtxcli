extern crate clap;
extern crate directories;

use clap::{App, Arg};
use directories::ProjectDirs;

// example of conditional compilation


#[cfg(all(unix, target_os="linux"))]
const OS: &'static str= "Linux";

#[cfg(windows)]
const OS: &'static str= "Windows";

#[cfg(all(unix, target_os="macos"))]
const OS: &'static str= "MacOS";

#[cfg(not(any(unix, windows)))]
const OS: &'static str= "Unknown";

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
    println!("OS: {}", OS);
}
