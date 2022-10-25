use std::io::Error;

use crate::mtxcli::interactive::{ShellCmdApi,Interactive};
use crate::{cmd_api,cmd_help};

#[derive(Debug)]
pub struct Status {
}
impl Status {
    pub fn new() -> Self {
        Status {
        }
    }
}

impl<'a> ShellCmdApi<'a> for Status {
    cmd_api!(status);

    cmd_help!("/status");

    fn process(&self, _args: &str, env: &mut Interactive, _commands: &Vec<Box<dyn ShellCmdApi>>) -> Result<bool, Error> {
        env.mtxcli.prompt();
        if env.mtxcli.logged_in {
            print!("status: logged in. ");
        } else {
            print!("status: not connected. ");
        }
        println!("username = {}, server = {}", env.mtxcli.username, env.mtxcli.server);
        Ok(false)
    }
}
