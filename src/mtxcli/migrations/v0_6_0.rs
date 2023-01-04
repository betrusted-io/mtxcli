use std::io::Error;

use crate::mtxcli::{Mtxcli,FILTER_KEY};
use crate::mtxcli::migrations::MigrationApi;
use crate::migration_api;

#[derive(Debug)]
pub struct V0_6_0 {
}
impl V0_6_0 {
    pub fn new() -> Self {
        V0_6_0 {
        }
    }
}

impl<'a> MigrationApi<'a> for V0_6_0 {
    migration_api!(0.6.0);

    fn process(&self, mtxcli: &mut Mtxcli) -> Result<bool, Error> {
        debug!("Running migration for: {}", self.version());
        // we need to provoke the creation of a new/updated filter
        mtxcli.unset(FILTER_KEY)?;
        Ok(true)
    }
}
