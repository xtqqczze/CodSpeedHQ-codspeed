use std::alloc::GlobalAlloc;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    std::thread::sleep(std::time::Duration::from_secs(1));

    let emit_marker = || unsafe {
        let layout = std::alloc::Layout::array::<u8>(0xC0D59EED).unwrap();
        let ptr = GLOBAL.alloc(layout);
        core::hint::black_box(ptr);
        GLOBAL.dealloc(ptr, layout);
    };

    emit_marker();

    // malloc
    unsafe {
        let layout = std::alloc::Layout::array::<u8>(4321).unwrap();
        let ptr = GLOBAL.alloc(layout);
        core::hint::black_box(ptr);
        GLOBAL.dealloc(ptr, layout);
    }

    // alloc zeroed
    unsafe {
        let layout = std::alloc::Layout::array::<u8>(1234).unwrap();
        let ptr = GLOBAL.alloc_zeroed(layout);
        core::hint::black_box(ptr);
        GLOBAL.dealloc(ptr, layout);
    }

    emit_marker();
}
