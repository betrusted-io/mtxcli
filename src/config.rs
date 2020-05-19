//! Program configuration
//!
//! This module initializes logging and configuration as well as
//! dispatches program actions.

use std::fmt;
use serde_json::{Value as Json, Map, json};
use clap::{App, Arg};
use std::fs::{File,OpenOptions};
use std::io::prelude::*;
// use mkdirp;
use std::path::PathBuf;
use regex::Regex;

mod system;
use system::System;
mod logger;
mod show;
mod parking;

/// The configuration filename
const CONFIG_FILE: &str = "config.json";

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

fn print_json(name: &str, v: &Json) {
    // let s = v.to_string();
    let s = serde_json::to_string_pretty(v).unwrap();
    println!("{} is a {}: {}", name, json_type_str(v), s);
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
    pub fn new(qualifier: &'static str, organization: &'static str,
               app: &'static str, version: &'static str,
               config: &'a mut Map<String,Json>) -> Self {
        let system = System::new(qualifier, organization, app);
        let mut config_path: PathBuf = PathBuf::from(system.config_dir.clone());
        config_path.push(CONFIG_FILE);
        // let json = json!({"config": config_path.to_str().unwrap()});
        //let json2 = json.clone();
        // print_json("config_json", &json2);
        // let config_json: &'a mut Json = &mut json;
        // let config: &'a mut Map<String,Json> = config_json.as_object_mut().unwrap();
        // let mut map = Map::new();
        // let config: &'a mut Map<String,Json> = &mut map;
        config.insert("config".to_string(), Json::String(config_path.to_str().unwrap().to_string()));
        let action = Action::Default; // FIXME
        Config {
            qualifier,
            organization,
            app,
            version,
            system,
            action,
            // config_json,
            config
        }
    }

    pub fn assign_json(&mut self, k: &str, v: Json) {
        match self.config.get_mut(k) {
            Some(e) => *e = v,
            None => {
                self.config.insert(k.to_string(), v);
                ()
            }
        }
    }

    pub fn assign(&mut self, k: &str, v: &str) {
        self.assign_json(k, json!(v));
    }

    pub fn set_jsons(&mut self, m: &Map<String,Json>) {
        let config_filename = self.get("config");
        println!("SET reading from: {}", config_filename);
        let mut config_str = String::new();
        match File::open(&config_filename)
            .and_then(|mut f| f.read_to_string(&mut config_str)) {
                Ok(_) => (),
                // verify the error is
                // Os { code: 2, kind: NotFound, message: "No such file or directory" }
                // Err(e) => println!("could not open {}: {:?}", config_filename, e)
                Err(_) => ()
            }
        if config_str.len() == 0 {
            config_str.push_str("{}");
        }
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

                let mut new_str = serde_json::to_string_pretty(&config_json).unwrap();
                new_str.push('\n'); // UNIXism
                println!("new_str: {}", new_str);
                let mut config_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(config_filename)
                    .expect("unable to open config file");
                config_file.write_all(new_str.as_bytes())
                    .expect("unable to write");
            }
        } else {
            println!("NOT HANDLED, not json object");
        }
    }

    /*
    pub fn sets(&mut self, ks: &Vec<String>, vs: &Vec<String>) {
        assert_eq!(ks.len(), vs.len());
        let mut rks: Vec<&str> = Vec::new();
        let mut js: Vec<Json> = Vec::new();
        let mut rjs: Vec<&Json> = Vec::new();
        for i in 0..ks.len() {
            rks.push(&ks[i]);
            js.push(Json::String(vs[i].to_string()));
            rjs.push(&js[i]);
        }
        self.set_jsons(&rks, &rjs);
    }
     */

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

    /// get var
    pub fn get(&mut self, k: &str) -> String {
        match self.config.get(k) {
            Some(e) => {
                if e.is_string() {
                    return e.as_str().unwrap().to_string();
                } else {
                    println!("{} is not a string, it's a {}", k, json_type_str(e));
                    return serde_json::to_string(e).unwrap();
                }
            },
            None => ()
        }
        "".to_string()
    }

    /// parse command line arguments
    pub fn parse_args(&mut self) {
        let keqv = Regex::new(r"([^=]+)=([^=]+)").unwrap();
        /*
        mkdirp::mkdirp(&system.config_dir)
            .expect("could not create config directory");
         */
        logger::init(&self.system.config_dir, "trace");
        trace!("starting {} =========", self.app);
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
            .arg(Arg::with_name("parking-lot")
                 .short("P")
                 .long("parking-lot")
                 .help("Parking Lot function"))
            .arg(Arg::with_name("logging-demo")
                 .short("L")
                 .long("logging-demo")

                 .help("Demonstrates all the log levels"))
            .get_matches();
        if let Some(config) = matches.value_of("config") {
            println!("config: {}", config); // FIXME
            self.assign("config", config);

        }
        if let Some(sets) = matches.values_of("set") {
            let mut m: Map<String,Json> = Map::new();
            for s in sets {
                match keqv.captures(s) {
                    Some(cap) => {
                        println!("set: {} = {}", &cap[1], &cap[2]);
                        m.insert(cap[1].to_string(), json!(cap[2]));
                    },
                    None => {
                        println!("set malformed: {}", s);
                    }
                }
            }
            self.set_jsons(&m);
        }
        if let Some(assigns) = matches.values_of("assign") {
            for a in assigns {
                match keqv.captures(a) {
                    Some(cap) => {
                        println!("assign: {} = {}", &cap[1], &cap[2]);
                        self.assign(&cap[1], &cap[2]);
                    },
                    None => {
                        println!("assign malformed: {}", a);
                    }
                }
            }
        }
        if let Some(gets) = matches.values_of("get") {
            for g in gets {
                println!("get: {} = {}", g, self.get(g));
            }
        }
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

    /// do the actions, return the exit code
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
// impl fmt::Display for Config {
impl fmt::Display for Config<'_> {
    // fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} from {}.{}\nsystem: {}",
               self.app, self.version,
               self.organization, self.qualifier,
               self.system)
    }
}
