//! Viewport-sized single-canvas layout, scrolling and retained demo contexts.
use std::cell::RefCell;
#[derive(Default)]
struct Page {
    items: Vec<(u32, usize, f32)>,
    slots: Vec<f32>,
    groups: Vec<f32>,
    positions: Vec<f32>,
    width: f32,
    height: f32,
    offset: f32,
    total: f32,
    rail: f32,
    expanded: bool,
    menu: bool,
    physics: bhoo::scroll::ScrollPhysics,
    drawer_motion: bhoo::timing::Tween,
    drawer_progress: f32,
    dragging: bool,
    last: f64,
    thumb: bool,
    thumb_grab: f32,
    rail_motion: bhoo::timing::Tween,
    reduced: bool,
    motion_initialized: bool,
}
thread_local! { static PAGE:RefCell<Page> = RefCell::new(Page{expanded:true,..Page::default()}); }
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_add(scene: u32, group: u32, height: f32) {
    PAGE.with_borrow_mut(|p| p.items.push((scene, group as usize, height)));
}
#[allow(clippy::too_many_arguments)]
fn slot(p: &mut Page, id: u32, scene: u32, x: f32, y: f32, w: f32, h: f32, clip: [f32; 4]) {
    if y + h > clip[1] && y < clip[1] + clip[3] {
        p.slots.extend([
            id as f32,
            scene as f32,
            x,
            y,
            w,
            h,
            clip[0],
            clip[1],
            clip[2],
            clip[3],
        ]);
    }
}
fn layout(p: &mut Page) {
    p.slots.clear();
    p.groups.clear();
    p.positions.resize(p.items.len(), 0.);
    let (w, h) = (p.width, p.height);
    let mobile = w < 600.;
    let target = if mobile {
        0.
    } else if p.expanded {
        256.
    } else {
        88.
    };
    if !p.motion_initialized {
        p.rail_motion = bhoo::timing::Tween::at(target);
        p.motion_initialized = true;
    }
    let spec = bhoo::timing::TweenSpec::new(0.3, bhoo::timing::Easing::Standard);
    let rail = p.rail_motion.sample(p.last, target, spec, spec, p.reduced);
    p.rail = rail;
    p.drawer_progress = p
        .drawer_motion
        .sample(p.last, f32::from(p.menu), spec, spec, p.reduced);
    let clip = [
        rail,
        0.,
        (w - rail - 10.).max(1.),
        if mobile { h - 80. } else { h },
    ];
    let hero = h.max(620.);
    slot(p, 3, 10002, rail, -p.offset, w - rail, hero, clip);
    let left = rail + 24.;
    let cw = (w - rail - 58.).max(100.);
    let cols = if cw > 1400. {
        3
    } else if cw > 800. {
        2
    } else {
        1
    };
    let cell = (cw - 24. * (cols - 1) as f32) / cols as f32;
    let mut y = hero + 48.;
    for group in 0..42 {
        p.groups.push(y);
        slot(
            p,
            10 + group as u32,
            10003,
            left,
            y - p.offset,
            cw,
            96.,
            clip,
        );
        y += 96.;
        let indices: Vec<usize> = p
            .items
            .iter()
            .enumerate()
            .filter(|(_, v)| v.1 == group)
            .map(|(i, _)| i)
            .collect();
        for row in indices.chunks(cols) {
            let rh = row.iter().map(|&i| p.items[i].2 + 88.).fold(0., f32::max);
            for (col, &i) in row.iter().enumerate() {
                let (scene, _, height) = p.items[i];
                p.positions[i] = y;
                let x = left + col as f32 * (cell + 24.);
                slot(
                    p,
                    1000 + i as u32 * 2,
                    10003,
                    x,
                    y - p.offset,
                    cell,
                    88.,
                    clip,
                );
                slot(
                    p,
                    1001 + i as u32 * 2,
                    scene,
                    x,
                    y + 88. - p.offset,
                    cell,
                    height,
                    clip,
                );
            }
            y += rh + 24.;
        }
        y += 48.;
    }
    p.total = y + 80.;
    slot(
        p,
        1,
        10000,
        if mobile {
            -280. * (1. - p.drawer_progress)
        } else {
            0.
        },
        0.,
        if mobile { 280. } else { rail },
        h,
        if mobile && p.drawer_progress <= 0. {
            [0., 0., 0., 0.]
        } else {
            [0., 0., w, h]
        },
    );
    slot(
        p,
        2,
        10001,
        rail + (w - rail - 256.) / 2.,
        12.,
        256.,
        64.,
        [0., 0., w, h],
    );
    if mobile {
        slot(p, 4, 10005, 0., h - 80., w, 80., [0., 0., w, h]);
    }
    // Drawer stays above floating chrome.
    if mobile && p.drawer_progress > 0. {
        let pos = p
            .slots
            .as_chunks::<10>()
            .0
            .iter()
            .position(|s| s[0] == 1.)
            .unwrap();
        let row = p.slots.drain(pos * 10..pos * 10 + 10).collect::<Vec<_>>();
        p.slots.extend(row);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_update(
    w: f32,
    h: f32,
    time: f64,
    x: f32,
    y: f32,
    flags: u32,
    key: u32,
    delta: f32,
    capture: u32,
) -> u32 {
    PAGE.with_borrow_mut(|p| {
        p.width = w.max(100.);
        p.height = h.max(100.);
        p.last = time;
        let max = (p.total - h).max(0.);
        p.physics.offset = p.offset;
        if flags & (1 | 4 | 8 | 16) != 0 || delta != 0. {
            p.physics.cancel();
        }
        if flags & 1 != 0 {
            p.physics.cancel();
            p.dragging = false;
            p.thumb = x > w - 14. && !p.menu;
            if p.thumb {
                let track = if w < 600. { h - 80. } else { h };
                let size = (track * track / p.total.max(track)).max(32.);
                let top = p.offset / max.max(1.) * (track - size);
                p.thumb_grab = if y >= top && y <= top + size {
                    y - top
                } else {
                    size / 2.
                };
            }
            if flags & 128 != 0 && capture == 0 && x > p.rail && !p.menu && !p.thumb {
                p.physics.begin(y, time);
            }
        }
        if p.thumb && flags & 4 == 0 {
            let track = if w < 600. { h - 80. } else { h };
            let size = (track * track / p.total.max(track)).max(32.);
            p.physics.offset = ((y - p.thumb_grab) / (track - size).max(1.) * max).clamp(0., max);
            p.dragging = true;
        }
        if p.physics.touching() && y.is_finite() {
            p.physics.drag(y, time, max, h);
            p.dragging |= p.physics.dragging;
        }
        if flags & 2 != 0 {
            p.physics.release(time, p.reduced);
            p.thumb = false;
        }
        if flags & (4 | 8) != 0 {
            p.physics.cancel();
            p.thumb = false;
        }
        if delta != 0. && x >= p.rail && !p.menu {
            p.physics.wheel(delta, max, p.reduced);
        }
        if flags & 16 != 0 && !p.menu && capture == 0 {
            p.physics.cancel();
            match key {
                12 => p.physics.wheel(-h * 0.85, max, p.reduced),
                13 => p.physics.wheel(h * 0.85, max, p.reduced),
                8 => p.physics.offset = 0.,
                9 => p.physics.offset = max,
                6 => p.physics.wheel(-48., max, p.reduced),
                7 => p.physics.wheel(48., max, p.reduced),
                _ => {}
            }
        }
        let scrolling = p.physics.advance(time, max, p.reduced);
        p.offset = p.physics.offset;
        layout(p);
        let suppressed = p.dragging;
        if flags & (2 | 4 | 8) != 0 {
            p.dragging = false;
        }
        u32::from(suppressed)
            | (u32::from(scrolling || p.rail_motion.moving() || p.drawer_motion.moving()) << 1)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_action(action: u32) {
    PAGE.with_borrow_mut(|p| {
        let previous = p.offset;
        p.physics.cancel();
        match action {
            1..=42 => {
                p.offset = p.groups.get(action as usize - 1).copied().unwrap_or(0.) - 88.;
                p.menu = false;
            }
            1000 | 1008 => {
                p.offset = 0.;
                p.menu = false;
            }
            1002 => {
                if p.width < 600. {
                    p.menu = false;
                } else {
                    p.expanded = !p.expanded;
                }
            }
            1003 => {
                if p.width < 600. {
                    p.menu = !p.menu;
                } else {
                    p.expanded = !p.expanded;
                }
            }
            1005 => p.offset = p.groups.first().copied().unwrap_or(0.) - 88.,
            1009 => p.offset = (p.total - p.height).max(0.),
            _ => {}
        }
        if p.offset != previous {
            let target = p.offset;
            let max = (p.total - p.height).max(0.);
            p.offset = previous;
            p.physics.offset = previous;
            p.physics.scroll_to(target, max, p.last, 0.5, p.reduced);
            p.offset = p.physics.offset;
        }
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_jump(action: u32) {
    bhoo_page_action(action);
    PAGE.with_borrow_mut(|p| {
        p.physics.finish_scroll_to();
        p.offset = p.physics.offset;
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_slots() -> u32 {
    PAGE.with_borrow(|p| p.slots.as_ptr() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_slots_len() -> u32 {
    PAGE.with_borrow(|p| p.slots.len() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_offset() -> f32 {
    PAGE.with_borrow(|p| p.offset)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_total() -> f32 {
    PAGE.with_borrow(|p| p.total)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_expanded() -> u32 {
    PAGE.with_borrow(|p| u32::from(p.expanded))
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_menu() -> u32 {
    PAGE.with_borrow(|p| u32::from(p.menu))
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_item(index: u32) {
    PAGE.with_borrow_mut(|p| {
        p.physics.cancel();
        p.offset = p.positions.get(index as usize).copied().unwrap_or(0.) - 88.;
        p.menu = false;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn reset() {
        PAGE.with_borrow_mut(|p| {
            *p = Page {
                expanded: true,
                ..Page::default()
            }
        });
        for group in 0..42 {
            bhoo_page_add(1, group, 384.);
        }
        bhoo_page_update(1200., 800., 0., 900., 500., 0, 0, 0., 0);
    }
    #[test]
    fn wheel_and_keyboard_scroll_clamp_to_document() {
        reset();
        bhoo_page_update(1200., 800., 0.1, 900., 500., 0, 0, 600., 0);
        assert!(bhoo_page_offset() > 0. && bhoo_page_offset() < 600.);
        for i in 1..120 {
            bhoo_page_update(1200., 800., 0.1 + i as f64 / 120., 900., 500., 0, 0, 0., 0);
        }
        assert!((bhoo_page_offset() - 600.).abs() < 0.1);
        bhoo_page_update(1200., 800., 0.2, 900., 500., 16, 9, 0., 0);
        assert_eq!(bhoo_page_offset(), bhoo_page_total() - 800.);
        bhoo_page_update(1200., 800., 0.3, 900., 500., 16, 8, 0., 0);
        assert_eq!(bhoo_page_offset(), 0.);
        // Wheel events in the rail must not also move the document.
        bhoo_page_update(1200., 800., 0.4, 100., 500., 0, 0, 600., 0);
        assert_eq!(bhoo_page_offset(), 0.);
    }
    #[test]
    fn touch_drag_cancels_clicks_and_continues_with_inertia() {
        reset();
        bhoo_page_update(1200., 800., 0.1, 900., 600., 129, 0, 0., 0);
        let flags = bhoo_page_update(1200., 800., 0.13, 900., 300., 128, 0, 0., 0);
        assert_ne!(flags & 1, 0);
        assert_eq!(bhoo_page_offset(), 300.);
        bhoo_page_update(1200., 800., 0.16, 900., 300., 130, 0, 0., 0);
        let released = bhoo_page_offset();
        bhoo_page_update(1200., 800., 0.19, 900., 300., 0, 0, 0., 0);
        assert!(bhoo_page_offset() > released);
        bhoo_page_update(1200., 800., 0.20, 900., 300., 4, 0, 0., 0);
        let stopped = bhoo_page_offset();
        bhoo_page_update(1200., 800., 0.23, 900., 300., 0, 0, 0., 0);
        assert_eq!(stopped, bhoo_page_offset());
    }
    #[test]
    fn holding_finger_before_release_stops_fling() {
        reset();
        bhoo_page_update(1200., 800., 0.1, 900., 600., 129, 0, 0., 0);
        bhoo_page_update(1200., 800., 0.13, 900., 300., 128, 0, 0., 0);
        bhoo_page_update(1200., 800., 0.4, 900., 300., 130, 0, 0., 0);
        let stopped = bhoo_page_offset();
        bhoo_page_update(1200., 800., 0.42, 900., 300., 0, 0, 0., 0);
        assert_eq!(stopped, bhoo_page_offset());
    }

    #[test]
    fn captured_widget_gesture_does_not_scroll_document() {
        reset();
        bhoo_page_update(1200., 800., 0.1, 900., 600., 129, 0, 0., 1);
        bhoo_page_update(1200., 800., 0.13, 900., 300., 128, 0, 0., 1);
        assert_eq!(bhoo_page_offset(), 0.);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_selected() -> u32 {
    PAGE.with_borrow(|p| {
        p.groups
            .iter()
            .rposition(|&y| y <= p.offset + 100.)
            .map_or(0, |i| i as u32 + 1)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_reduced(value: u32) {
    PAGE.with_borrow_mut(|p| {
        p.reduced = value != 0;
    });
}

#[cfg(test)]
mod rail_motion_tests {
    use super::*;
    #[test]
    fn rail_width_interpolates_reverses_without_jumping_and_respects_reduced_motion() {
        PAGE.with_borrow_mut(|p| {
            *p = Page {
                expanded: true,
                ..Page::default()
            }
        });
        bhoo_page_update(1200., 800., 0., 900., 500., 0, 0, 0., 0);
        bhoo_page_action(1002);
        bhoo_page_update(1200., 800., 0.1, 900., 500., 0, 0, 0., 0);
        bhoo_page_update(1200., 800., 0.2, 900., 500., 0, 0, 0., 0);
        let middle = PAGE.with_borrow(|p| p.rail);
        assert!(middle > 88. && middle < 256.);
        bhoo_page_action(1002);
        bhoo_page_update(1200., 800., 0.2, 900., 500., 0, 0, 0., 0);
        assert_eq!(PAGE.with_borrow(|p| p.rail), middle);
        bhoo_page_update(1200., 800., 0.6, 900., 500., 0, 0, 0., 0);
        assert_eq!(PAGE.with_borrow(|p| p.rail), 256.);
        bhoo_page_reduced(1);
        bhoo_page_action(1002);
        bhoo_page_update(1200., 800., 0.7, 900., 500., 0, 0, 0., 0);
        assert_eq!(PAGE.with_borrow(|p| p.rail), 88.);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_page_drawer_progress() -> f32 {
    PAGE.with_borrow(|p| p.drawer_progress)
}
