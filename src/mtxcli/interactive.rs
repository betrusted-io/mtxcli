//! Interactive Action
//!
//! Enters interactive mode

use std::io;
use std::io::{Error,prelude::*};
use std::boxed::Box;

use crate::mtxcli::Mtxcli;

mod get;      use get::*;
mod help;     use help::*;
mod login;    use login::*;
mod logout;   use logout::*;
mod quit;     use quit::*;
mod set;      use set::*;
mod status;   use status::*;
mod unset;    use unset::*;


/// Interactive struct
// #[derive(Debug)]
pub struct Interactive<'a> {
    mtxcli: &'a mut Mtxcli,
}

/// implementation of Interactive
impl<'a> Interactive<'a> {

    /// Construct a new Interactive
    pub fn new(mtxcli: &'a mut Mtxcli) -> Self {
        Interactive {
            mtxcli,
        }
    }

    fn run(&mut self) -> Result<(), Error> {
        let mut commands: Vec<Box<dyn ShellCmdApi>> = Vec::new();
        commands.push(Box::new(Get::new()));
        commands.push(Box::new(Help::new()));
        commands.push(Box::new(Login::new()));
        commands.push(Box::new(Logout::new()));
        commands.push(Box::new(Quit::new()));
        commands.push(Box::new(Set::new()));
        commands.push(Box::new(Status::new()));
        commands.push(Box::new(Unset::new()));
        if self.mtxcli.args.verbose > 0 {
            println!("{} interactive", self.mtxcli.app);
        }
        let mut quit = false;
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let mut cmdline = line?;
            let maybe_verb = tokenize(&mut cmdline);
            if let Some(verb) = maybe_verb {
                // if verb starts with a slash then it's a command (else chat)
                if verb.starts_with("/") {
                    // search through the list of commands linearly until one
                    // matches, then run it.
                    let command = &verb[1..];
                    let mut match_found = false;
                    for cmd in commands.iter() {
                        if !match_found && cmd.matches(command) {
                            match_found = true;
                            quit = cmd.process(&cmdline, self, &commands)?;
                        };
                    }
                    // if none match, create a list of available commands
                    if !match_found {
                        let mut first = true;
                        print!("Commands: ");
                        for cmd in commands.iter() {
                            if !first {
                                print!(", ");
                            }
                            print!("/{}", cmd.verb());
                            first = false;
                        }
                        println!();
                    }
                } else { // chat
                    self.mtxcli.user_says(&format!("{} {}", verb, cmdline));
                }
            } else { // no verb
                self.mtxcli.user_says("");
            }
            if quit {
                break;
            }
        }
        Ok(())
    }
}

pub trait ShellCmdApi<'a> {

    fn process(&self, args: &str, env: &mut Interactive, commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error>;

    // created with cmd_api! macro
    // checks if the command matches the current verb in question
    fn matches(&self, verb: &str) -> bool;
    // returns my verb
    fn verb(&self) -> &'static str;
    // returns my help
    fn help(&self) -> &'static str;
}

// the argument to this macro is the command verb
#[macro_export]
macro_rules! cmd_api {
    ($verb:expr) => {
        fn verb(&self) -> &'static str {
            stringify!($verb)
        }
        fn matches(&self, verb: &str) -> bool {
            if verb == stringify!($verb) {
                true
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! cmd_help {
    ($verb:expr) => {
        fn help(&self) -> &'static str {
            $verb
        }
    };
}

/// Interactive Mode
pub fn act(mtxcli: &mut Mtxcli) -> i32  {
    let mut interactive = Interactive::new(mtxcli);
    match interactive.run() {
        Ok(()) => {
            0
        },
        Err(e) => {
            error!("interactive error: {:?}", e);
            1
        }
    }
}

/// extract the first token, as delimited by spaces
/// modifies the incoming line by removing the token and returning the remainder
/// returns the found token
pub fn tokenize(line: &mut String) -> Option<String> {
    let mut token = String::new();
    let mut retline = String::new();

    let lineiter = line.chars();
    let mut foundspace = false;
    let mut foundrest = false;
    for ch in lineiter {
        if ch != ' ' && !foundspace {
            token.push(ch);
        } else if foundspace && foundrest {
            retline.push(ch);
        } else if foundspace && ch != ' ' {
            // handle case of multiple spaces in a row
            foundrest = true;
            retline.push(ch);
        } else {
            foundspace = true;
            // consume the space
        }
    }
    line.clear();
    line.push_str(&retline);
    if token.len() > 0 {
        Some(token)
    } else {
        None
    }
}
