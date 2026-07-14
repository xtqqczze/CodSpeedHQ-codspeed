use super::MemtrackBpf;
use crate::prelude::*;
use libbpf_rs::MapCore;

impl MemtrackBpf {
    pub fn add_tracked_pid(&mut self, pid: i32) -> Result<()> {
        self.skel
            .maps
            .tracked_pids
            .update(
                &pid.to_le_bytes(),
                &1u8.to_le_bytes(),
                libbpf_rs::MapFlags::ANY,
            )
            .context("Failed to add PID to uprobes tracked set")?;

        Ok(())
    }

    pub fn enable_tracking(&mut self) -> Result<()> {
        let key = 0u32;
        let value = true as u8;
        self.skel
            .maps
            .tracking_enabled
            .update(
                &key.to_le_bytes(),
                &value.to_le_bytes(),
                libbpf_rs::MapFlags::ANY,
            )
            .context("Failed to enable tracking")?;
        Ok(())
    }

    pub fn disable_tracking(&mut self) -> Result<()> {
        let key = 0u32;
        let value = false as u8;
        self.skel
            .maps
            .tracking_enabled
            .update(
                &key.to_le_bytes(),
                &value.to_le_bytes(),
                libbpf_rs::MapFlags::ANY,
            )
            .context("Failed to disable tracking")?;
        Ok(())
    }

    pub fn dropped_events_count(&self) -> Result<u64> {
        let key = 0u32;
        let value = self
            .skel
            .maps
            .dropped_events
            .lookup(&key.to_le_bytes(), libbpf_rs::MapFlags::ANY)
            .context("Failed to read dropped_events counter")?
            .ok_or_else(|| anyhow!("dropped_events slot 0 missing"))?;

        let bytes: [u8; 8] = value
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("dropped_events value has unexpected size"))?;
        Ok(u64::from_le_bytes(bytes))
    }
}
