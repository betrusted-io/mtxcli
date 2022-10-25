use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Set {
}
impl Set {
    pub fn new() -> Self {
        Set {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Set {
    cmd_api!(set);

    cmd_help!("/set key value");

    fn process(&self, args: &str, env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        let mut tokens = args.split(' ');
        if let Some(key) = tokens.next() {
            match key {
                "" => {
                    env.mtxcli.prompt();
                    println!("{}", self.help());
                }
                _ => {
                    if let Some(value) = tokens.next() {
                        match value {
                            "" => {
                                env.mtxcli.prompt();
                                println!("{}", self.help());
                           }
                            _ => {
                                match env.mtxcli.set(key, value) {
                                    Ok(()) => {
                                        // println!("# set {} = {}", key, value);
                                    },
                                    Err(e) => {
                                        error!("error setting key {}: {:?}", key, e);
                                    }
                                }
                            }
                        }
                    } else {
                        env.mtxcli.prompt();
                        println!("{}", self.help());
                    }
                }
            }
        }

        Ok(false)
    }
}
