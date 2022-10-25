use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Quit {
}
impl Quit {
    pub fn new() -> Self {
        Quit {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Quit {
    cmd_api!(quit);

    cmd_help!("/quit");

    fn process(&self, _args: &str, _env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        Ok(true)
    }
}
