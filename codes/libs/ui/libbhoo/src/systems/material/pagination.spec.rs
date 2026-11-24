use crate::*;

#[test]
fn paginator_formats_range_and_navigates_pages() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 80.0 };
    let mut page = 0usize;
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.paginator(2000, &mut page, 10, 25, true);
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "1 – 10 of 25")));
    let next = ui.semantics().iter().find(|n| n.label.as_ref() == "Next page").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([next.x + 8.0, next.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.paginator(2000, &mut page, 10, 25, true);
        });
    }
    assert_eq!(page, 1);
}

#[test]
fn pagination_options_paginator_page_size_and_page_indicator_work() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 480.0, height: 220.0 };
    let mut p = 5usize;
    let _ = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.pagination_with_options(
            10,
            &mut p,
            20,
            true,
            PaginationOptions { boundary_count: 1, sibling_count: 1, show_first_last: true },
        );
    });
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "First page"));
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Last page"));

    let mut page = 0usize;
    let mut page_size = 10usize;
    let _ = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.paginator_with_page_size(50, &mut page, &mut page_size, &[10, 25, 50], 100, true);
    });
    let sz_btn = ui.semantics().iter().find(|n| n.label.as_ref() == "Items per page: 10").unwrap().bounds;
    let pt = [sz_btn.x + 12.0, sz_btn.y + 12.0];
    for (pressed, released, time) in [(true, false, 0.1), (false, true, 0.15)] {
        ui.frame(Input { pointer: Some(pt), pressed, released, time, ..Input::default() }, bounds, 8.0, |f| {
            f.paginator_with_page_size(50, &mut page, &mut page_size, &[10, 25, 50], 100, true);
        });
    }
    assert_eq!(page_size, 25);

    let dot_rect = Rect { x: 100.0, y: 120.0, width: 80.0, height: 32.0 };
    let mut switched = None;
    for (pressed, released, time) in [(true, false, 0.2), (false, true, 0.25)] {
        ui.frame(Input { pointer: Some([140.0, 136.0]), pressed, released, time, ..Input::default() }, bounds, 8.0, |f| {
            switched = f.page_indicator(90, dot_rect, 3, 0.0, PageIndicatorOptions { interactive: true, ..PageIndicatorOptions::default() });
        });
    }
    assert_eq!(switched, Some(1));
}
