//! Program configuration
//!
//! This module initializes logging and configuration as well as
//! dispatches program actions.

use clap::{App, Arg};
use regex::Regex;
use serde_json::{Value as Json, Map, json};
use std::fmt;
use std::path::PathBuf;

mod system;
use system::System;
mod logger;
mod show;
mod parking;

/// The configuration filename
const CONFIG_FILE: &str = "config.json";

/// The configuration filename key
const CONFIG_FILE_KEY: &str = "config";

/// Empty JSON object
const EMPTY_OBJECT: &str = "{}";

/// The log level key
const LEVEL_KEY: &str = "global_level";

/// Default logging level
const LEVEL_DEFAULT: &str = "trace";

/// Returns the datatype of the Json value
pub fn json_type_str(v: &Json) -> &'static str {
    match *v {
        Json::Null => "null",
        Json::Bool(..) => "boolean",
        Json::Number(..) => "number",
        Json::String(..) => "string",
        Json::Array(..) => "array",
        Json::Object(..) => "object",
    }
}

/// Pretty prints the Json value with the given label
fn print_json(label: &str, v: &Json) {
    let s = serde_json::to_string_pretty(v).unwrap();
    println!("{} is a {}: {}", label, json_type_str(v), s);
}

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
    /// Logging Demo
    LoggingDemo,
    /// Parking Lot
    ParkingLot,
}

/// Config struct
#[derive(Debug, PartialEq)]
pub struct Config<'a> {
    pub qualifier: &'static str,
    pub organization: &'static str,
    pub app: &'static str,
    pub version: &'static str,
    pub system: System,
    pub action: Action,
    config: &'a mut Map<String,Json>,
}

/// implementation of Config
impl<'a> Config<'a> {

    /// Construct a new Config
    pub fn new(qualifier: &'static str, organization: &'static str,
               app: &'static str, version: &'static str,
               config: &'a mut Map<String,Json>) -> Self {
        let system = System::new(qualifier, organization, app);
        let mut config_path: PathBuf = PathBuf::from(system.config_dir.clone());
        config_path.push(CONFIG_FILE);
        config.insert(CONFIG_FILE_KEY.to_string(),
                      Json::String(config_path.to_str().unwrap().to_string()));
        config.insert(LEVEL_KEY.to_string(), Json::String(LEVEL_DEFAULT.to_string()));
        let action = Action::Default;
        Config {
            qualifier,
            organization,
            app,
            version,
            system,
            action,
            config
        }
    }

    /// Assign the configuration key **k** the Json value **v**
    pub fn assign_json(&mut self, k: &str, v: Json) {
        match self.config.get_mut(k) {
            Some(e) => *e = v,
            None => { self.config.insert(k.to_string(), v); () }
        }
    }

    /// Assign the configuration key **k** the value **v**
    pub fn assign(&mut self, k: &str, v: &str) {
        self.assign_json(k, json!(v));
    }

    /// Read the config file from the filesystem
    fn read_config (&mut self) {
        let config_filename = self.get(CONFIG_FILE_KEY);
        println!("read_config from: {}", config_filename);
        let mut config_str = String::new();
        self.system.read_file_to_str(&mut config_str, &config_filename, EMPTY_OBJECT);
        let mut config_json: Json = serde_json::from_str(&config_str).unwrap();
        if config_json.is_object() {
            print_json("read from file", &config_json);
            let config_map = config_json.as_object_mut().unwrap();
            for (k, v) in config_map {
                self.assign_json(k, v.clone());
            }
        } else {
            error!("configurion file is not a json object");
        }
    }

    /// Set multiple Json values
    pub fn set_jsons(&mut self, m: &Map<String,Json>) {
        let config_filename = self.get(CONFIG_FILE_KEY);
        println!("read_config from: {}", config_filename);
        let mut config_str = String::new();
        self.system.read_file_to_str(&mut config_str, &config_filename, EMPTY_OBJECT);
        let mut mutations = 0;
        let mut config_json: Json = serde_json::from_str(&config_str).unwrap();
        if config_json.is_object() {
            print_json("read from file", &config_json);
            let config_map = config_json.as_object_mut().unwrap();
            for (k, v) in m {
                self.assign_json(k, v.clone());
                match config_map.get_mut(k) {
                    Some(e) => if e != v {
                        *e = v.clone();
                        mutations += 1;
                    },
                    None => {
                        config_map.insert(k.to_string(), v.clone());
                        mutations += 1;
                    }
                }
            }
            println!("mutations: {}", mutations);
            if mutations > 0 {
                let str = serde_json::to_string_pretty(&config_json)
                    .expect("unable to serialize config");
                self.system.write_file_from_str(&str, &config_filename)
                    .expect("unable to write config file");
            }
        } else {
            error!("configurion file is not a json object");
        }
    }

    /*
    pub fn set_json(&mut self, k: &str, v: &Json) {
        let mut ks: Vec<&str> = Vec::new();
        ks.push(k);
        let mut vs: Vec<&Json> = Vec::new();
        vs.push(v);
        self.set_jsons(&ks, &vs);
    }

    pub fn set(&mut self, k: &str, v: &str) {
        let j = json!(v);
        self.set_json(k, &j);
    }
     */

    /// Get a config value (with optional default, if not present)
    pub fn get_default(&self, k: &str, default: &str) -> String {
        if let Some(e) = self.config.get(k) {
            if e.is_string() {
                return e.as_str().unwrap().to_string();
            } else {
                debug!("{} is not a string, it's a {}", k, json_type_str(e));
                return serde_json::to_string(e).unwrap();
            }
        }
        default.to_string()
    }

    /// Get a config value (return the empty string if not present)
    pub fn get(&self, k: &str) -> String {
        self.get_default(k, system::EMPTY)
    }

    /// Parse command line arguments
    pub fn parse_args(&mut self) {
        let keqv = Regex::new(r"([^=]+)=([^=]+)").unwrap();
        let matches = App::new(self.app)
            .version(self.version)
            .about("Matrix Command Line Interface")
            .arg(Arg::with_name("assign")
                 .short("a")
                 .long("assign")
                 .value_name("var=value")
                 .help("sets var = value (not saved)")
                 .takes_value(true)
                 .multiple(true)
                 .number_of_values(1))
            .arg(Arg::with_name(CONFIG_FILE_KEY)
                 .short("c")
                 .long(CONFIG_FILE_KEY)
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
            .arg(Arg::with_name("parking-lot")
                 .short("P")
                 .long("parking-lot")
                 .help("Parking Lot function"))
            .arg(Arg::with_name("logging-demo")
                 .short("L")
                 .long("logging-demo")
                 .help("Demonstrates all the log levels"))
            .get_matches();
        if let Some(config) = matches.value_of(CONFIG_FILE_KEY) {
            self.assign(CONFIG_FILE_KEY, config);
        }
        self.read_config();
        if let Some(sets) = matches.values_of("set") {
            let mut m: Map<String,Json> = Map::new();
            for s in sets {
                match keqv.captures(s) {
                    Some(cap) => {
                        debug!("set: {} = {}", &cap[1], &cap[2]);
                        m.insert(cap[1].to_string(), json!(cap[2]));
                    },
                    None => {
                        error!("set malformed: {}", s);
                    }
                }
            }
            self.set_jsons(&m);
        }
        if let Some(assigns) = matches.values_of("assign") {
            for a in assigns {
                match keqv.captures(a) {
                    Some(cap) => {
                        debug!("assign: {} = {}", &cap[1], &cap[2]);
                        self.assign(&cap[1], &cap[2]);
                    },
                    None => {
                        error!("assign malformed: {}", a);
                    }
                }
            }
        }
        if let Some(gets) = matches.values_of("get") {
            for g in gets {
                println!("{} = {}", g, self.get(g));
            }
        }
        logger::init(&self.system.config_dir, &self.get(LEVEL_KEY));
        trace!("starting {} =========", self.app);
        self.action = if matches.is_present("show-config") {
            Action::ShowConfig
        } else if matches.is_present("parking-lot")  {
            Action::ParkingLot
        } else if matches.is_present("logging-demo")  {
            Action::LoggingDemo
        } else {
            Action::Default
        };
    }

    /// Do the action, return the exit code
    pub fn act(&mut self) -> i32 {
        trace!("action: {:?}", self.action);
        match self.action {
            Action::ShowConfig => show::act(self),
            Action::LoggingDemo => logger::act(self),
            Action::ParkingLot => parking::act(self),
            _ =>  0
        }
    }
}

/// simple Display for Config
impl fmt::Display for Config<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut a = String::new();
        a.push_str(self.app);
        a.push(' ');
        a.push_str(self.version);
        a.push_str(" from ");
        a.push_str(self.organization);
        a.push('.');
        a.push_str(self.qualifier);
        a.push_str(&self.system.eol);
        a.push_str("system: ");
        let mut b = String::new();
        b.push_str(&self.system.eol);
        b.push_str("-- config --");
        for k in self.config.keys() {
            b.push_str(&self.system.eol);
            b.push_str(k);
            b.push_str(": ");
            if let Some(v) = self.config.get(k) {
                b.push_str(&serde_json::to_string_pretty(v).unwrap())
            }
        }
        write!(f, "{}{}{}", a, self.system, b)
    }
}
