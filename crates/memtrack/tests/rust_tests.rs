#[macro_use]
mod shared;

use memtrack::AllocatorLib;
use rstest::rstest;
use std::path::Path;

#[test_with::env(GITHUB_ACTIONS)]
#[rstest]
#[case("system", &[])]
#[case("jemalloc", &["with-jemalloc"])]
#[case("mimalloc", &["with-mimalloc"])]
#[test_log::test]
fn test_rust_alloc_tracking(
    #[case] name: &str,
    #[case] features: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let crate_path = Path::new("testdata/alloc_rust");
    let binary = shared::compile_rust_binary(crate_path, "alloc_rust", features)?;

    // Try to find a static allocator in the binary, then attach to it as well
    // This is needed because the CWD is different, which breaks the heuristics.
    let allocators = AllocatorLib::from_path_static(&binary)
        .map(|a| vec![a])
        .unwrap_or_default();

    let (events, thread_handle) = shared::track_binary_with_opts(&binary, &allocators)?;
    assert_events_with_marker!(name, &events);

    thread_handle.join().unwrap();
    Ok(())
}
