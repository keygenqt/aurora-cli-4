use std::error::Error;

use crate::tools::emulator::Emulator;

pub struct EmulatorRunPackage {}

impl EmulatorRunPackage {
    pub fn exec(package: String) -> Result<(), Box<dyn Error>> {
        let emulator = Emulator::default();
        emulator.run(&package)?;
        Ok(())
    }
}
