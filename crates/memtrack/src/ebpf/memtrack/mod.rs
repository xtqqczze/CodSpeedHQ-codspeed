use crate::prelude::*;
use libbpf_rs::Link;
use libbpf_rs::skel::OpenSkel;
use libbpf_rs::skel::SkelBuilder;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::path::Path;

use crate::ebpf::poller::RingBufferPoller;

pub mod memtrack_skel {
    include!(concat!(env!("OUT_DIR"), "/memtrack.skel.rs"));
}
pub use memtrack_skel::*;

#[macro_use]
mod macros;
mod allocator;
mod maps;
mod tracking;

/// Resolve libbpf attach targets for every defined symbol in `lib_path`.
pub fn resolve_symbol_offsets(lib_path: &Path) -> Result<ResolvedSymbols> {
    use object::{Object, ObjectSymbol};

    let data = std::fs::read(lib_path)?;
    let file = object::File::parse(&*data)?;
    let mut offsets = HashMap::new();

    for symbol in file.symbols().chain(file.dynamic_symbols()) {
        if !symbol.is_definition() {
            continue;
        }

        let Ok(name) = symbol.name() else {
            continue;
        };

        if let Some(file_offset) = symbol_file_offset(&file, &symbol) {
            offsets.insert(name.to_owned(), file_offset);
        }
    }

    Ok(ResolvedSymbols { offsets })
}

/// The libbpf file offset for `symbol`, or `None` when it has no address in a
/// file-backed section (absolute, `SHT_NOBITS`, ...).
fn symbol_file_offset<'a>(
    file: &object::File,
    symbol: &impl object::ObjectSymbol<'a>,
) -> Option<usize> {
    use object::{Object, ObjectSection};

    let address = symbol.address();
    if address == 0 {
        return None;
    }

    let section = file.section_by_index(symbol.section_index()?).ok()?;
    let (sh_offset, _) = section.file_range()?;
    Some((address - section.address() + sh_offset) as usize)
}

/// Attach targets resolved from a library's symbol tables.
pub struct ResolvedSymbols {
    offsets: HashMap<String, usize>,
}

impl ResolvedSymbols {
    fn offset(&self, symbol: &str) -> Option<usize> {
        self.offsets.get(symbol).copied()
    }
}

pub struct MemtrackBpf {
    skel: Box<MainSkel<'static>>,
    probes: Vec<Link>,
}

impl MemtrackBpf {
    pub fn new() -> Result<Self> {
        let builder = MainSkelBuilder::default();
        let open_object = Box::leak(Box::new(MaybeUninit::uninit()));
        let open_skel = builder
            .open(open_object)
            .context("Failed to open syscalls BPF skeleton")?;

        let skel = Box::new(
            open_skel
                .load()
                .context("Failed to load syscalls BPF skeleton")?,
        );

        Ok(Self {
            skel,
            probes: Vec::new(),
        })
    }

    /// Start polling, forwarding each event over the returned channel.
    pub fn start_polling_with_channel(
        &self,
        poll_timeout_ms: u64,
    ) -> Result<(
        RingBufferPoller,
        crossbeam_channel::Receiver<runner_shared::artifacts::MemtrackEvent>,
    )> {
        RingBufferPoller::with_channel(&self.skel.maps.events, poll_timeout_ms)
    }

    /// Detach all BPF links in parallel. Closing a uprobe link blocks on two
    /// RCU grace periods in the kernel, but concurrent waiters share grace
    /// periods, so closing from many threads scales near-linearly.
    pub fn detach_probes(&mut self) {
        const DETACH_THREADS: usize = 32;

        let mut probes = std::mem::take(&mut self.probes);
        if probes.is_empty() {
            return;
        }

        debug!("Detaching {} BPF links", probes.len());
        let start = std::time::Instant::now();
        let chunk_size = probes.len().div_ceil(DETACH_THREADS);
        std::thread::scope(|scope| {
            while !probes.is_empty() {
                let split_at = probes.len().saturating_sub(chunk_size);
                let chunk = probes.split_off(split_at);
                scope.spawn(move || drop(chunk));
            }
        });
        debug!("Detached BPF links in {:?}", start.elapsed());
    }
}

impl Drop for MemtrackBpf {
    fn drop(&mut self) {
        self.detach_probes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allocator entry points must resolve to file offsets; a symbol that
    /// silently fails to resolve attaches nothing and loses all events.
    #[test]
    fn libc_allocator_symbols_resolve_to_offsets() {
        let maps = std::fs::read_to_string("/proc/self/maps").unwrap();
        let libc_path = maps
            .lines()
            .find_map(|line| {
                let path = line.split_whitespace().last()?;
                path.contains("libc.so.6").then(|| path.to_owned())
            })
            .expect("test process has no mapped libc.so.6");

        let symbols = resolve_symbol_offsets(Path::new(&libc_path)).unwrap();
        for symbol in ["malloc", "calloc", "realloc", "free"] {
            assert!(
                symbols.offset(symbol).is_some(),
                "{symbol} in {libc_path} did not resolve to a file offset"
            );
        }
    }
}
