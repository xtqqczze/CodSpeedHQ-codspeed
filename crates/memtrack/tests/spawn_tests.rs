#[macro_use]
mod shared;

use memtrack::AllocatorLib;
use memtrack::prelude::*;
use std::path::Path;
use std::process::Command;

fn compile_spawn_binary(name: &str) -> anyhow::Result<std::path::PathBuf> {
    shared::compile_rust_binary(Path::new("testdata/spawn_wrapper"), name, &[])
}

/// Without discovering the child's static allocator, the spawned child's
/// jemalloc allocations should NOT be tracked — we expect no marker-delimited events.
#[test_with::env(GITHUB_ACTIONS)]
#[test_log::test]
fn test_spawn_without_static_allocator_discovery() -> Result<(), Box<dyn std::error::Error>> {
    let wrapper = compile_spawn_binary("wrapper")?;
    let child = compile_spawn_binary("alloc_child")?;

    // No allocators attached — static jemalloc in alloc_child won't be found
    let mut cmd = Command::new(&wrapper);
    cmd.arg(&child);
    let (events, thread_handle) = shared::track_command(cmd, &[], false)?;

    assert_events_with_marker!("spawn_without_discovery", &events);

    thread_handle.join().unwrap();
    Ok(())
}

/// With the child binary's static allocator discovered (simulating
/// CODSPEED_MEMTRACK_BINARIES), allocations from the spawned child should be tracked.
#[test_with::env(GITHUB_ACTIONS)]
#[test_log::test]
fn test_spawn_with_static_allocator_discovery() -> Result<(), Box<dyn std::error::Error>> {
    let wrapper = compile_spawn_binary("wrapper")?;
    let child = compile_spawn_binary("alloc_child")?;

    // Simulate what CODSPEED_MEMTRACK_BINARIES does: discover the static jemalloc
    let allocator = AllocatorLib::from_path_static(&child)?;

    let mut cmd = Command::new(&wrapper);
    cmd.arg(&child);
    let (events, thread_handle) = shared::track_command(cmd, &[allocator], false)?;

    assert_events_with_marker!("spawn_with_discovery", &events);

    thread_handle.join().unwrap();
    Ok(())
}
