use super::MemtrackBpf;
use crate::prelude::*;
use paste::paste;

impl MemtrackBpf {
    attach_tracepoint!(sched_fork);

    pub fn attach_tracepoints(&mut self) -> Result<()> {
        self.attach_sched_fork()?;
        Ok(())
    }
}
