use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Unset {
}
impl Unset {
    pub fn new() -> Self {
        Unset {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Unset {
    cmd_api!(unset);

    cmd_help!("/unset key");

    fn process(&self, args: &str, env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        let mut tokens = args.split(' ');

        if let Some(key) = tokens.next() {
            match key {
                "" => {
                    env.mtxcli.prompt();
                    println!("{}", self.help());
                }
                _ => {
                    match env.mtxcli.unset(key) {
                        Ok(()) => {
                            // println!("unset {}", key);
                        },
                        Err(e) => {
                            error!("error unsetting key {}: {:?}", key, e);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}
