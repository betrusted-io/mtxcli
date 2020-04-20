//! Program configuration
//!
//! This module initializes logging and configuration as well as
//! dispatches program actions.

use std::fmt;
use clap::{App, Arg};

mod system;
use system::System;
mod logger;
mod show;
mod default;

/// Action for the program to act upon
#[derive(Debug, PartialEq)]
pub enum Action {
    // Show program version
    // Version,
    // Show usage
    // Help,
    /// default Action
    Default,
    /// Show configuration
    ShowConfig,
}

/// Config struct
#[derive(Debug, PartialEq)]
pub struct Config {
    pub qualifier: &'static str,
    pub organization: &'static str,
    pub app: &'static str,
    pub version: &'static str,
    pub system: System,
    pub action: Action,
}

/// implementation of Config
impl Config {
    pub fn new(qualifier: &'static str, organization: &'static str,
               app: &'static str, version: &'static str) -> Self {
        let system = System::new(qualifier, organization, app);
        logger::init(&system.config_dir, "trace");
        trace!("starting {} =====================================", app);
        let matches = App::new(app)
            .version(version)
            .about("Matrix Command Line Interface")
            .arg(Arg::with_name("assign")
                 .short("a")
                 .long("assign")
                 .value_name("var=value")
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
            .get_matches();
        if let Some(config) = matches.value_of("config") {
            println!("config: {}", config);
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
        let action = if matches.is_present("show-config") {
            Action::ShowConfig
        } else {
            Action::Default
        };
        Config {
            qualifier,
            organization,
            app,
            version,
            system,
            action
        }
    }

    /// do the actions, return the exit code
    pub fn act(&mut self) -> i32 {
        trace!("action: {:?}", self.action);
        match self.action {
            Action::ShowConfig => show::act(self),
            _ =>  default::act(self)
        }
    }
}

/// simple Display for Config
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} from {}.{}\nsystem: {}",
               self.app, self.version,
               self.organization, self.qualifier,
               self.system)
    }
}
