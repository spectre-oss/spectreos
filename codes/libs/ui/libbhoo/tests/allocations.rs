//! Count allocations on this test thread after warming the core's buffers/caches.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
struct Counting;
thread_local! { static ENABLED:Cell<bool>=const{Cell::new(false)}; static COUNT:Cell<usize>=const{Cell::new(0)}; static BYTES:Cell<usize>=const{Cell::new(0)}; }
// SAFETY: allocation/deallocation are delegated unchanged to the system allocator.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ENABLED.with(|enabled| {
            if enabled.get() {
                COUNT.with(|count| count.set(count.get() + 1));
                BYTES.with(|bytes| bytes.set(bytes.get() + layout.size()));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ENABLED.with(|enabled| {
            if enabled.get() {
                COUNT.with(|count| count.set(count.get() + 1));
                BYTES.with(|bytes| bytes.set(bytes.get() + size));
            }
        });
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
#[test]
fn warmed_controls_and_chip_transitions_allocate_nothing() {
    use bhoo::*;
    let mut ui = Ui::default();
    let mut commands = Vec::new();
    let mut value = false;
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 1000.0,
    };
    for _ in 0..3 {
        ui.frame_into(Input::default(), bounds, 8.0, &mut commands, |f| {
            for id in 1..10 {
                f.button(id, "Unchanged");
            }
            f.switch(100, "State", &mut value, true);
            f.chip(200, "Filter", true, true);
        });
    }
    COUNT.with(|x| x.set(0));
    ENABLED.with(|x| x.set(true));
    for i in 0..120 {
        ui.frame_into(
            Input {
                time: i as f64 / 120.0,
                ..Input::default()
            },
            bounds,
            8.0,
            &mut commands,
            |f| {
                for id in 1..10 {
                    f.button(id, "Unchanged");
                }
                f.switch(100, "State", &mut value, true);
                f.chip(200, "Filter", i / 15 % 2 == 0, true);
            },
        );
    }
    ENABLED.with(|x| x.set(false));
    let count = COUNT.with(Cell::get);
    assert_eq!(
        count, 0,
        "control build or chip transition allocated after warmup"
    );
}

#[test]
fn stack_view_frame_work_does_not_clone_a_large_adapter() {
    use bhoo::*;
    mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_stack_view.rs"));}
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let bounds=Rect{x:0.,y:0.,width:300.,height:220.};
    state.set_list_adapter("stack",(0..100_000).map(|i|format!("Item {i}")).collect(),None,None).unwrap();let mut commands=Vec::new();
    for _ in 0..3 {ui.frame_into(Input::default(),bounds,0.,&mut commands,|f|{fixture::LAYOUT.render(f,bounds,&mut state);});}
    COUNT.with(|count|count.set(0));ENABLED.with(|enabled|enabled.set(true));
    ui.frame_into(Input{time:1.,..Input::default()},bounds,0.,&mut commands,|f|{fixture::LAYOUT.render(f,bounds,&mut state);});
    ENABLED.with(|enabled|enabled.set(false));let count=COUNT.with(Cell::get);
    assert!(count<100,"one StackView frame allocated {count} times for a large adapter");
}

#[test]
fn large_position_based_adapter_does_not_allocate_a_row_sized_set() {
    let mut state=bhoo::android_xml::LayoutState::default();let rows=(0..100_000).map(|i|format!("Item {i}")).collect();
    BYTES.with(|bytes|bytes.set(0));ENABLED.with(|enabled|enabled.set(true));state.set_list_adapter("stack",rows,None,None).unwrap();ENABLED.with(|enabled|enabled.set(false));
    let bytes=BYTES.with(Cell::get);assert!(bytes<100_000,"adapter setup allocated {bytes} extra bytes");
}
