//! Program configuration
//!
//! This module initializes logging and configuration as well as
//! dispatches program actions.

use clap::{App, Arg};
use regex::Regex;
use serde_json::{Value as Json, Map, json};
use std::fmt;
use std::path::PathBuf;

mod interactive;
mod logger;
mod parking;
mod show;
mod system;
use system::System;
mod url;
mod datetime;

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
    pub prompt: String,
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
        let mut prompt = String::new();
        prompt.push('[');
        prompt.push_str(app);
        prompt.push(']');
        prompt.push(' ');
        Config {
            qualifier,
            organization,
            app,
            version,
            prompt,
            system,
            action,
            config
        }
    }

    /// Evaluates exp
    fn eval(&self, exp: &str) -> String {
        let mut value = String::new();
        let mut i;
        let mut j = 0;
        let re = Regex::new(r"(^|[^\\])(\$\[[A-Z0-9_]+\])") // TODO: cache this
            .expect("unable to compile env regex");
        for mat in re.find_iter(exp) {
            i = mat.start();
            if i > 0 || &exp[1..2] != "[" {
                i += 1;
            } // skip non backslash
            if i > j {
                value.push_str(&exp[j..i]);
            }
            j = mat.end();
            value.push_str(&self.system.getenv(&exp[i+2..j-1]));
        }
        if j < exp.len() {
            value.push_str(&exp[j..]);
        }
        let exp2 = &value.to_string();
        value.truncate(0);
        j = 0;
        let re2 = Regex::new(r"(^|[^\\])(\$\{[A-Za-z0-9_-]+\})") // TODO: cache this
            .expect("unable to compile variable regex");
        for mat2 in re2.find_iter(exp2) {
            i = mat2.start();
            if i > 0 || &exp2[1..2] != "{" {
                i += 1;
            }
            if i > j {
                value.push_str(&exp2[j..i]);
            }
            j = mat2.end();
            value.push_str(&self.get(&exp2[i+2..j-1]))
        }
        if j < exp2.len() {
            value.push_str(&exp2[j..]);
        }
        value
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
        let mut config_str = String::new();
        self.system.read_file_to_str(&mut config_str, &config_filename, EMPTY_OBJECT);
        let mut config_json: Json = serde_json::from_str(&config_str).unwrap();
        if config_json.is_object() {
            // print_json("read from file", &config_json);
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
        let mut config_str = String::new();
        self.system.read_file_to_str(&mut config_str, &config_filename, EMPTY_OBJECT);
        let mut mutations = 0;
        let mut config_json: Json = serde_json::from_str(&config_str).unwrap();
        if config_json.is_object() {
            // print_json("read from file", &config_json);
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

    /// Unset a key
    pub fn unset(&mut self, k: &str) {
        let config_filename = self.get(CONFIG_FILE_KEY);
        let mut config_str = String::new();
        self.system.read_file_to_str(&mut config_str, &config_filename, EMPTY_OBJECT);
        let mut mutations = 0;
        let mut config_json: Json = serde_json::from_str(&config_str).unwrap();
        if config_json.is_object() {
            // print_json("read from file", &config_json);
            let config_map = config_json.as_object_mut().unwrap();
            if config_map.contains_key(k) {
                config_map.remove(k);
                mutations += 1;
            }
            if self.config.contains_key(k) {
                self.config.remove(k);
            }
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

    pub fn set_json(&mut self, k: &str, v: Json) {
        let mut m: Map<String,Json> = Map::new();
        m.insert(k.to_string(), v);
        self.set_jsons(&m);
    }

    pub fn set(&mut self, k: &str, v: &str) {
        self.set_json(k, json!(v));
    }

    pub fn set_integer(&mut self, k: &str, v: i64) {
        self.set_json(k, json!(v));
    }

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

    /// Get a config value (with optional integer default, if not present)
    pub fn get_default_int(&self, k: &str, default: i64) -> i64 {
        if let Some(e) = self.config.get(k) {
            if e.is_i64() {
                return e.as_i64().unwrap();
            }
            if e.is_string() {
                if let Ok(v) = e.as_str().unwrap().parse::<i64>() {
                    return v;
                } else {
                    debug!("{} could not be converted to an integer", k);
                }
            } else {
                debug!("{} is not an integer, it's a {}", k, json_type_str(e));
            }
        }
        default
    }

    /// Get a config value (return the empty string if not present)
    pub fn get(&self, k: &str) -> String {
        self.get_default(k, system::EMPTY)
    }

    /// Get a config value returns true if set
    pub fn is(&self, k: &str) -> bool {
        &self.get(k) == "true"
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
            .arg(Arg::with_name("unset")
                 .short("u")
                 .long("unset")
                 .value_name("var")
                 .help("unsets var's value")
                 .takes_value(true)
                 .multiple(true)
                 .number_of_values(1))
            .arg(Arg::with_name("encode")
                 .short("e")
                 .long("encode")
                 .value_name("value")
                 .help("URL encodes value")
                 .takes_value(true)
                 .multiple(true)
                 .number_of_values(1))
            .arg(Arg::with_name("decode")
                 .short("d")
                 .long("decode")
                 .value_name("value")
                 .help("URL decodes value")
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
                        // println!("set: {} = {}", &cap[1], &cap[2]);
                        // m.insert(cap[1].to_string(), json!(cap[2]));
                        m.insert(self.eval(&cap[1]), json!(self.eval(&cap[2])));
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
                        // println!("assign: {} = {}", &cap[1], &cap[2]);
                        // self.assign(&cap[1], &cap[2]);
                        self.assign(&self.eval(&cap[1]), &self.eval(&cap[2]));
                    },
                    None => {
                        error!("assign malformed: {}", a);
                    }
                }
            }
        }
        if let Some(unsets) = matches.values_of("unset") {
            for u in unsets {
                self.unset(&self.eval(u));
            }
        }
        if let Some(gets) = matches.values_of("get") {
            for g in gets {
                let k = self.eval(g);
                println!("{} = {}", k, self.get(&k));
            }
        }
        if let Some(encodes) = matches.values_of("encode") {
            for e in encodes {
                println!("encode: {} = {}", e, url::encode(e));
            }
        }
        if let Some(decodes) = matches.values_of("decode") {
            for d in decodes {
                println!("decode: {} = {}", d, url::decode(d));
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
    pub fn act(&'a mut self) -> i32 {
        trace!("action: {:?}", self.action);
        match self.action {
            Action::ShowConfig => show::act(self),
            Action::LoggingDemo => logger::act(self),
            Action::ParkingLot => parking::act(self),
            _ =>  interactive::act(self)
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
