//! Migrations
//!
//! Runs migrations up to the current version

// use std::io;

use std::io::Error;
use crate::mtxcli::{Mtxcli,VERSION_KEY};

mod v0_5_0;      use v0_5_0::*;
mod v0_6_0;      use v0_6_0::*;

const DEFAULT_VERSION: &str = "0";

pub trait MigrationApi<'a> {
    // created with migration_api! macro
    // returns my version
    fn version(&self) -> &'static str;
    // checks if the migration should be applied
    fn applies(&self, version: &str) -> bool;
    // run the migration
    fn process(&self, mtxcli: &mut Mtxcli) -> Result<bool, Error>;
}

// the argument to this macro is the command verb
#[macro_export]
macro_rules! migration_api {
    ($version:expr) => {
        fn version(&self) -> &'static str {
            stringify!($version)
        }

        fn applies(&self, version: &str) -> bool {
            if version < self.version() {
                true
            } else {
                false
            }
        }
    };
}

/// Run migrations as needed
pub fn run_migrations(mtxcli: &mut Mtxcli) {
    let version = mtxcli.get_default(VERSION_KEY, DEFAULT_VERSION);
    if version.ne(mtxcli.version) {
        debug!("Running migrations from version {} to {}",
               version, mtxcli.version);
        let mut migrations: Vec<Box<dyn MigrationApi>> = Vec::new();
        migrations.push(Box::new(V0_5_0::new()));
        migrations.push(Box::new(V0_6_0::new()));
        for migration in migrations.iter() {
            if migration.applies(&version) {
                match migration.process(mtxcli) {
                    Ok(boolean) => {
                        debug!("migration competed: {}: {}",
                               migration.version(), boolean);
                        if boolean {
                            mtxcli.set(VERSION_KEY, migration.version())
                                .expect("cannot set _version");
                        }
                    },
                    Err(e) => {
                        error!("error running migration: {}: {:?}",
                               migration.version(), e);
                    }
                }
            }
        }
        mtxcli.set(VERSION_KEY, mtxcli.version)
            .expect("cannot set _version");
    }
}
