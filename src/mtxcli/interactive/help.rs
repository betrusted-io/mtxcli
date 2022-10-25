use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Help {
}
impl Help {
    pub fn new() -> Self {
        Help {
        }
    }
}

const UNKNOWN_HELP: &str = "unknown command";
const HELP_OVERVIEW: &str = "available mtxcli commands:";

impl<'a> ShellCmdApi<'a> for Help {
    cmd_api!(help);

    cmd_help!("/help [cmd]");

    fn process(&self, args: &str, env: &mut Interactive, commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        let mut tokens = args.split(' ');
        if let Some(slashcmd) = tokens.next() {
            let command = if slashcmd.starts_with("/") {
                &slashcmd[1..]
            } else {
                slashcmd
            };
            match command {
                "" => {
                    env.mtxcli.prompt();
                    println!("{}", HELP_OVERVIEW);
                    for cmd in commands.iter() {
                        env.mtxcli.prompt();
                        println!("{}", cmd.help());
                    }
                }
                _ => {
                    let mut match_found = false;
                    for cmd in commands.iter() {
                        if !match_found && cmd.matches(command) {
                            match_found = true;
                            env.mtxcli.prompt();
                            println!("{}", cmd.help());
                        };
                    }
                    if !match_found {
                        env.mtxcli.prompt();
                        println!("{}: {}", UNKNOWN_HELP, command);
                    }
                }
            }
        }
        Ok(false)
    }
}
