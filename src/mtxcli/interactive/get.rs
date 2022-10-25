use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Get {
}
impl Get {
    pub fn new() -> Self {
        Get {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Get {
    cmd_api!(get);

    cmd_help!("/get key");

    fn process(&self, args: &str, env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        let mut tokens = args.split(' ');

        if let Some(key) = tokens.next() {
            match key {
                "" => {
                    env.mtxcli.prompt();
                    println!("{}", self.help());

                }
                _ => {
                    match env.mtxcli.get(key) {
                        Ok(None) => {
                            env.mtxcli.prompt();
                            println!("{} is UNSET", key);
                        },
                        Ok(Some(value)) => {
                            env.mtxcli.prompt();
                            println!("{}", value);
                        }
                        Err(e) => {
                            error!("error getting key {}: {:?}", key, e);
                        }
                    }
                }
            }
        }
        Ok(false)
    }
}
