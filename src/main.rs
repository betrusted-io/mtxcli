extern crate clap;
extern crate directories;

use clap::{App, Arg};
use directories::ProjectDirs;

/// The main **mtxcli** program
fn main() {
    let matches = App::new("mtxcli")
        .version("0.2.0")
        .author("Tom Marble <tmarble@info9.net>")
        .about("Matrix Command Line Interface")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets a custom config file")
             .takes_value(true))
        .arg(Arg::with_name("show-config")
                .short("s")
                .long("show-config")
                .help("Show config file path"))
        .get_matches();
    let proj_dirs = ProjectDirs::from("io", "Betrusted", "mtxcli")
        .expect("could not find default config directory");
    let config_default = proj_dirs.config_dir().to_str()
        .expect("could not convert config_dir to a string");
    let config_dir = matches.value_of("config")
        .unwrap_or_else(|| config_default);

    if matches.is_present("show-config") {
        println!("config: {}", config_dir);
    }
    println!("Welcome to mtxcli!");
}
