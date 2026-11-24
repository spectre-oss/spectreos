use crate::*;

#[test]
fn data_table_sort_header_cycles_asc_desc_none() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 300.0 };
    let cols = [
        TableColumn { label: "Name", weight: 0.6, sortable: true, align_end: false },
        TableColumn { label: "Size", weight: 0.4, sortable: true, align_end: true },
    ];
    let mut sort = TableSortState::default();
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.data_table(300, &cols, 2, &mut sort, |r, c| format!("{r}:{c}"));
    });
    let hdr = ui.semantics().iter().find(|n| n.id == 300).unwrap().bounds;
    for expected in [SortDirection::Ascending, SortDirection::Descending, SortDirection::None] {
        for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
            ui.frame(Input { pointer: Some([hdr.x + 10.0, hdr.y + 10.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
                f.data_table(300, &cols, 2, &mut sort, |r, c| format!("{r}:{c}"));
            });
        }
        assert_eq!(sort.direction, expected);
    }
}
