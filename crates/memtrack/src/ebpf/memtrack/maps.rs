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

    /// Mark a (dev, ino) as classified so the watcher stops re-signalling for it.
    /// The 16-byte key matches `struct inode_key { __u64 dev; __u64 ino; }` (no padding).
    pub fn insert_known_inode(&self, dev: u64, ino: u64) -> Result<()> {
        let mut key = [0u8; 16];
        key[..8].copy_from_slice(&dev.to_le_bytes());
        key[8..].copy_from_slice(&ino.to_le_bytes());
        self.skel
            .maps
            .known_inodes
            .update(&key, &1u8.to_le_bytes(), libbpf_rs::MapFlags::ANY)
            .context("Failed to insert known inode")?;
        Ok(())
    }

    /// Number of exec-mapping requests dropped because the request ring buffer was full.
    pub fn attach_request_dropped_count(&self) -> Result<u64> {
        let key = 0u32;
        let value = self
            .skel
            .maps
            .attach_request_dropped
            .lookup(&key.to_le_bytes(), libbpf_rs::MapFlags::ANY)
            .context("Failed to read attach_request_dropped counter")?
            .ok_or_else(|| anyhow!("attach_request_dropped slot 0 missing"))?;

        let bytes: [u8; 8] = value
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("attach_request_dropped value has unexpected size"))?;
        Ok(u64::from_le_bytes(bytes))
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
