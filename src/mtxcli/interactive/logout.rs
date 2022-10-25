use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Logout {
}
impl Logout {
    pub fn new() -> Self {
        Logout {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Logout {
    cmd_api!(logout);

    cmd_help!("/logout");

    fn process(&self, _args: &str, env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        env.mtxcli.logout();
        Ok(false)
    }
}
