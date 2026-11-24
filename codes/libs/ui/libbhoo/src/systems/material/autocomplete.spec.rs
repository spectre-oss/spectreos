use crate::*;

#[test]
fn autocomplete_filters_and_selects_suggestion() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 360.0 };
    let opts = ["Display", "Sound", "Storage", "System"];
    let mut open = true;
    let mut chosen = None;
    ui.frame(Input::default(), bounds, 4.0, |f| {
        let (_, c) = f.autocomplete(800, "Setting", "st", &opts, &mut open, AutocompleteOptions::default());
        chosen = c;
    });
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Storage"));
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "System"));
    assert!(ui.semantics().iter().all(|n| n.label.as_ref() != "Display"));

    let sto = ui.semantics().iter().find(|n| n.label.as_ref() == "Storage").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([sto.x + 8.0, sto.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 4.0, |f| {
            let (_, c) = f.autocomplete(800, "Setting", "st", &opts, &mut open, AutocompleteOptions::default());
            if c.is_some() {
                chosen = c;
            }
        });
    }
    assert_eq!(chosen, Some(2));
    assert!(!open);
}
