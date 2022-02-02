//! Performance profiling.

use crate::*;

impl Cx {
    pub fn profile_start(&mut self, id: u64) {
        self.profiles.insert(id, UniversalInstant::now());
    }

    pub fn profile_end(&self, id: u64) {
        if let Some(inst) = self.profiles.get(&id) {
            log!("Profile {} time {}ms", id, inst.elapsed().as_millis());
        }
    }
}
