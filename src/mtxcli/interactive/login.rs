use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Login {
}
impl Login {
    pub fn new() -> Self {
        Login {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Login {
    cmd_api!(login);

    cmd_help!("/login");

    fn process(&self, _args: &str, env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        env.mtxcli.login();
        Ok(false)
    }
}
