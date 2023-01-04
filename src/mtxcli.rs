//! Program configuration
//!
//! This module initializes logging and configuration as well as
//! dispatches program actions.

use std::fmt;
use std::fs::File;
use std::io::{Read, Write, Error, ErrorKind};
use std::path::PathBuf;

use clap::Parser;

mod interactive;
mod migrations;  use migrations::run_migrations;
mod parking;
mod system;      use system::System;
mod url;
mod web;

const FILTER_KEY: &str = "_filter";
const PASSWORD_KEY: &str = "password";
const ROOM_ID_KEY: &str = "_room_id";
const ROOM_KEY: &str = "room";
const SINCE_KEY: &str = "_since";
const SERVER_KEY: &str = "server";
const TOKEN_KEY: &str = "_token";
const USER_KEY: &str = "user";
const USERNAME_KEY: &str = "username";
const VERSION_KEY: &str = "_version";
const CURRENT_VERSION_KEY: &str = "__version";

const HTTPS: &str = "https://";
const SERVER_MATRIX: &str = "https://matrix.org";

const EMPTY: &str = "";
const MTX_TIMEOUT: i32 = 300; // ms

#[derive(Parser,Default,Debug,PartialEq)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    parking: bool,
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
    /// Parking Lot
    ParkingLot,
}

/// Config struct
#[derive(Debug, PartialEq)]
pub struct Mtxcli {
    pub qualifier: &'static str,
    pub organization: &'static str,
    pub app: &'static str,
    pub version: &'static str,
    pub system: System,
    pub args: Args,
    pub action: Action,
    pub user: String,
    pub username: String,
    pub server: String,
    token: String,
    pub logged_in: bool,
    pub room_id: String,
    pub filter: String,
    pub since: String,
}

/// implementation of Mtxcli
impl Mtxcli {

    /// Construct a new Mtxcli
    pub fn new(qualifier: &'static str, organization: &'static str,
               app: &'static str, version: &'static str) -> Self {
        let system = System::new(qualifier, organization, app);
        let action = Action::Default;
        let args = Args::parse();
        let logged_in = false;
        Mtxcli {
            qualifier,
            organization,
            app,
            version,
            system,
            args,
            action,
            user: EMPTY.to_string(),
            username: USER_KEY.to_string(),
            server: SERVER_MATRIX.to_string(),
            token: EMPTY.to_string(),
            logged_in,
            room_id: EMPTY.to_string(),
            filter: EMPTY.to_string(),
            since: EMPTY.to_string(),
        }
    }

    /// Parse command line arguments
    pub fn parse_args(&mut self) {
        flexi_logger::Logger::try_with_env().unwrap()
            .start()
            .expect("cannot initialize flexi_logger");
        self.args = Args::parse();
        self.action = if self.args.parking  {
            Action::ParkingLot
        } else {
            Action::Default
        };
        run_migrations(self);
    }

    /// Do the action, return the exit code
    pub fn act(&mut self) -> i32 {
        trace!("action: {:?}", self.action);
        self.user = self.get_default(USER_KEY, EMPTY);
        self.username = self.get_default(USERNAME_KEY, USERNAME_KEY);
        self.server = self.get_default(SERVER_KEY, SERVER_MATRIX);
        self.room_id = self.get_default(ROOM_ID_KEY, EMPTY);
        self.filter = self.get_default(FILTER_KEY, EMPTY);
        self.since = self.get_default(SINCE_KEY, EMPTY);
        match self.action {
            Action::ParkingLot => parking::act(self),
            _ =>  interactive::act(self)
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), Error> {
        if key.starts_with("__") {
            Err(Error::new(ErrorKind::PermissionDenied,
                           "may not set a variable beginning with __ "))
        } else {
            let mut keypath = PathBuf::new();
            keypath.push(&self.system.config_dir);
            std::fs::create_dir_all(&keypath)?;
            keypath.push(key);
            File::create(keypath)?.write_all(value.as_bytes())?;
            match key { // special case side effects
                USER_KEY => { self.set_user(value); }
                PASSWORD_KEY => { self.set_password(); }
                ROOM_KEY => { self.set_room(); }
                _ => { }
            }
            Ok(())
        }
    }

    pub fn set_user(&mut self, value: &str) {
        debug!("# USER_KEY set '{}' = '{}'", USER_KEY, value);
        let i = match value.find('@') {
            Some(index) => { index + 1 },
            None => { 0 },
        };
        let j = match value.find(':') {
            Some(index) => { index },
            None => { value.len() },
        };
        self.username = (&value[i..j]).to_string();
        if j < value.len() {
            self.server = String::from(HTTPS);
            self.server.push_str(&value[j + 1..]);
        } else {
            self.server = SERVER_MATRIX.to_string();
        }
        self.user = value.to_string();
        debug!("# user = '{}' username = '{}' server = '{}'", self.user, self.username, self.server);
        self.set(USERNAME_KEY, &self.username.clone()).unwrap();
        self.set(SERVER_KEY, &self.server.clone()).unwrap();
        self.unset(TOKEN_KEY).unwrap();
        self.token = EMPTY.to_string();
    }

    pub fn set_password(&mut self) {
        debug!("# PASSWORD_KEY set '{}' => clearing TOKEN_KEY", PASSWORD_KEY);
        self.unset(TOKEN_KEY).unwrap();
    }

    pub fn set_room(&mut self) {
        debug!("# ROOM_KEY set '{}' => clearing ROOM_ID_KEY, SINCE_KEY, FILTER_KEY", ROOM_KEY);
        self.unset(ROOM_ID_KEY).unwrap();
        self.unset(SINCE_KEY).unwrap();
        self.unset(FILTER_KEY).unwrap();
    }

    pub fn unset(&mut self, key: &str) -> Result<(), Error> {
        if key.starts_with("__") {
            Err(Error::new(ErrorKind::PermissionDenied,
                           "may not unset a variable beginning with __ "))
        } else {
            let mut keypath = PathBuf::new();
            keypath.push(&self.system.config_dir);
            std::fs::create_dir_all(&keypath)?;
            keypath.push(key);
            if std::fs::metadata(&keypath).is_ok() { // keypath exists
                std::fs::remove_file(keypath)?;
            }
            Ok(())
        }
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>, Error> {
        if key.eq(CURRENT_VERSION_KEY) {
            Ok(Some(self.version.to_string()))
        } else {
            let mut keypath = PathBuf::new();
            keypath.push(&self.system.config_dir);
            std::fs::create_dir_all(&keypath)?;
            keypath.push(key);
            if let Ok(mut file)= File::open(keypath) {
                let mut value = String::new();
                file.read_to_string(&mut value)?;
                Ok(Some(value))
            } else {
                Ok(None)
            }
        }
    }

    pub fn get_default(&mut self, key: &str, default: &str) -> String {
        match self.get(key) {
            Ok(None) => {
                default.to_string()
            },
            Ok(Some(value)) => {
                value.to_string()
            }
            Err(e) => {
                error!("error getting key {}: {:?}", key, e);
                default.to_string()
            }
        }
    }

    pub fn prompt(&self) {
        print!("{}> ", self.app);
    }

    pub fn user_says(&mut self, text: &str) {
        if ! self.logged_in {
            if ! self.login() {
                self.prompt();
                println!("error: not connected");
                return;
            }
        }
        if self.room_id.len() == 0 {
            if ! self.get_room_id() {
                self.prompt();
                println!("error: could not find room_id");
                return;
            }
        }
        if self.filter.len() == 0 {
            if ! self.get_filter() {
                self.prompt();
                println!("error: could not create filter");
                return;
            }
        }
        self.read_messages();
        if text.len() > 0 {
            if web::send_message(&self.server, &self.room_id, &text, &self.token) {
                // The following is not required, because we will get what
                // the user said when we read_messages
                // println!("{}> {}", self.username, text);
                self.read_messages(); // update since to include what user said
            } else {
                println!("{}> {} # FAILED TO SEND", self.username, text);
            }
        } // else just update
    }

    pub fn login(&mut self) -> bool {
        self.prompt();
        println!("logging in...");
        self.token = self.get_default(TOKEN_KEY, EMPTY);
        self.logged_in = false;
        if self.token.len() > 0 {
            if web::whoami(&self.server, &self.token) {
                self.logged_in = true;
            }
        }
        if ! self.logged_in {
            if web::get_login_type(&self.server) {
                let user = self.get_default(USER_KEY, USER_KEY);
                let password = self.get_default(PASSWORD_KEY, EMPTY);
                if password.len() == 0 {
                    self.prompt();
                    println!("please /set user @USER:matrix.org");
                    self.prompt();
                    println!("please /set password my-password");
                } else {
                    if let Some(new_token) = web::authenticate_user(&self.server, &user, &password) {
                        self.set(TOKEN_KEY, &new_token).unwrap();
                        self.token = new_token;
                        self.logged_in = true;
                    } else {
                        error!("Error: may NOT login with type: {}", web::MTX_LOGIN_PASSWORD);
                    }
                }
            }
        }
        self.prompt();
        if self.logged_in {
            println!("logged in");
        } else {
            println!("authentication failed");
        }
        self.logged_in
    }

    pub fn logout(&mut self) {
        self.unset(TOKEN_KEY).unwrap();
        self.prompt();
        println!("logged out");
        self.logged_in = false;
    }

    // assume logged in, token is valid
    pub fn get_room_id(&mut self) -> bool {
        if self.room_id.len() > 0 {
            true
        } else {
            let room = self.get_default(ROOM_KEY, EMPTY);
            let server = self.get_default(SERVER_KEY, EMPTY);
            if room.len() == 0 {
                self.prompt();
                println!("please /set room my-room-to-join");
                false
            } else if server.len() == 0 {
                self.prompt();
                println!("please /set server my-matrix-server");
                false
            } else {
                let mut room_server = String::new();
                if ! room.starts_with("#") {
                    room_server.push_str("#");
                }
                room_server.push_str(&room);
                room_server.push_str(":");
                let i = match server.find(HTTPS) {
                    Some(index) => {
                        index + HTTPS.len()
                    },
                    None => {
                        server.len()
                    },
                };
                if i >= server.len() {
                    self.prompt();
                    println!("please /set server my-matrix-server (INVALID)");
                    false
                } else {
                    room_server.push_str(&server[i..]);
                    if let Some(new_room_id) = web::get_room_id(&self.server, &room_server, &self.token) {
                        self.set(ROOM_ID_KEY, &new_room_id).unwrap();
                        self.room_id = new_room_id;
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }

    // assume logged in, token is valid, room_id is valid, user is valid
    pub fn get_filter(&mut self) -> bool {
        if self.filter.len() > 0 {
            true
        } else {
            if let Some(new_filter) = web::get_filter(&self.user, &self.server,
                                                      &self.room_id, &self.token) {
                self.set(FILTER_KEY, &new_filter).unwrap();
                self.filter = new_filter;
                true
            } else {
                false
            }
        }
    }

    // assume logged in, token is valid, room_id is valid, user is valid,
    // and filter is valid
    pub fn read_messages(&mut self) {
        if let Some((since, messages)) = web::client_sync(&self.server, &self.filter,
                                                          &self.since, MTX_TIMEOUT,
                                                          &self.room_id, &self.token) {
            self.set(SINCE_KEY, &since).unwrap();
            self.since = since;
            debug!("since = {}", self.since);
            if messages.len() > 0 {
                print!("{}", messages);
            }
        }
    }
}

/// simple Display for Mtxcli
impl fmt::Display for Mtxcli {
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
        write!(f, "{}{}{}", a, self.system, b)
    }
}
