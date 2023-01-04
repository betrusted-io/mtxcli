use std::io::Error;

use crate::mtxcli::Mtxcli;
use crate::mtxcli::migrations::MigrationApi;
use crate::migration_api;

#[derive(Debug)]
pub struct V0_5_0 {
}
impl V0_5_0 {
    pub fn new() -> Self {
        V0_5_0 {
        }
    }
}

impl<'a> MigrationApi<'a> for V0_5_0 {
    migration_api!(0.5.0);


    fn process(&self, mtxcli: &mut Mtxcli) -> Result<bool, Error> {
        debug!("Running migration for: {}", self.version());
        debug!("This is just a test migration for {}", mtxcli.organization);
        Ok(false)
    }
}
