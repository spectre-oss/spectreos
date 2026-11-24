//! Wave 13B core rendering & ABI foundation tests:
//! - TextStyle `italic` and `family` (`FontFamily`)
//! - `DrawCommand::PushTransform` / `PopTransform` and `Frame` helpers
//! - `DrawCommand::PushLayer` / `PopLayer` and `Frame` helpers
//! - `RelativeRule.kind: u16` (>= 256) with zero struct-size growth
use bhoo::{
    android_xml::{
        LayoutState, Node, RULE_ELEVATION, RULE_RANGE_SLIDER, RelativeRule, Size,
    },
    *,
};

#[path = "../../../../../apps/bhoo-web/src/android_textappearance.rs"]
mod android_textappearance;

#[test]
fn text_style_defaults_and_variants() {
    let default = TextStyle::default();
    assert!(!default.italic);
    assert_eq!(default.family, FontFamily::Sans);
    assert_eq!(FontFamily::default(), FontFamily::Sans);

    assert!(!TextStyle::LABEL.italic);
    assert_eq!(TextStyle::LABEL.family, FontFamily::Sans);

    let italic_serif = TextStyle {
        size: 28.0,
        weight: 700,
        italic: true,
        family: FontFamily::Serif,
        ..TextStyle::default()
    };
    let mono = TextStyle {
        size: 13.0,
        weight: 500,
        italic: false,
        family: FontFamily::Monospace,
        ..TextStyle::default()
    };

    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 120.0,
    };
    let commands = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.styled_text(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 40.0,
            },
            "Serif Italic",
            Color(255, 255, 255, 255),
            italic_serif,
        );
        f.styled_text(
            Rect {
                x: 0.0,
                y: 48.0,
                width: 200.0,
                height: 30.0,
            },
            "0xCAFE",
            Color(200, 220, 255, 255),
            mono,
        );
    });

    let texts: Vec<_> = commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { text, style, .. } => Some((text.as_ref(), *style)),
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].0, "Serif Italic");
    assert!(texts[0].1.italic);
    assert_eq!(texts[0].1.family, FontFamily::Serif);
    assert_eq!(texts[0].1.weight, 700);
    assert_eq!(texts[1].0, "0xCAFE");
    assert!(!texts[1].1.italic);
    assert_eq!(texts[1].1.family, FontFamily::Monospace);
}

#[test]
fn frame_emits_push_and_pop_transform() {
    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 240.0,
    };
    let commands = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.push_transform(
            [160.0, 120.0],
            [12.0, -8.0],
            [1.25, 0.75],
            std::f32::consts::FRAC_PI_4,
        );
        f.paint_rect(
            Rect {
                x: 100.0,
                y: 80.0,
                width: 120.0,
                height: 80.0,
            },
            12.0,
            Color(120, 80, 220, 255),
        );
        f.pop_transform();
    });

    assert!(commands.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform {
            origin: [ox, oy],
            translate: [tx, ty],
            scale: [sx, sy],
            rotation_rad,
        } if (*ox - 160.0).abs() < 1e-6
            && (*oy - 120.0).abs() < 1e-6
            && (*tx - 12.0).abs() < 1e-6
            && (*ty + 8.0).abs() < 1e-6
            && (*sx - 1.25).abs() < 1e-6
            && (*sy - 0.75).abs() < 1e-6
            && (*rotation_rad - std::f32::consts::FRAC_PI_4).abs() < 1e-6
    )));
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::PushTransform { .. }))
            .count(),
        1
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::PopTransform))
            .count(),
        1
    );
}

#[test]
fn frame_emits_push_and_pop_layer_with_clamped_alpha() {
    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 240.0,
    };
    let layer_rect = Rect {
        x: 20.0,
        y: 20.0,
        width: 180.0,
        height: 120.0,
    };
    let commands = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.push_layer(layer_rect, 0.5);
        f.paint_rect(layer_rect, 8.0, Color(255, 0, 0, 255));
        f.push_layer(layer_rect.inset(10.0), 1.75);
        f.paint_rect(layer_rect.inset(10.0), 4.0, Color(0, 0, 255, 255));
        f.pop_layer();
        f.pop_layer();
        f.push_layer(layer_rect, -0.25);
        f.pop_layer();
    });

    let pushed: Vec<(Rect, f32)> = commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::PushLayer { bounds, alpha } => Some((*bounds, *alpha)),
            _ => None,
        })
        .collect();
    assert_eq!(pushed.len(), 3);
    assert_eq!(pushed[0], (layer_rect, 0.5));
    assert_eq!(pushed[1], (layer_rect.inset(10.0), 1.0));
    assert_eq!(pushed[2], (layer_rect, 0.0));
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::PopLayer))
            .count(),
        3
    );
}

#[test]
fn relative_rule_kind_supports_u16_values_without_struct_growth() {
    assert_eq!(
        std::mem::size_of::<RelativeRule>(),
        std::mem::size_of::<(u8, &'static str)>(),
        "Widening RelativeRule.kind from u8 to u16 must not grow RelativeRule"
    );

    static HIGH_RULES: &[RelativeRule] = &[
        RelativeRule {
            kind: 256,
            anchor: "extended_256",
        },
        RelativeRule {
            kind: 512,
            anchor: "extended_512",
        },
        RelativeRule {
            kind: u16::MAX,
            anchor: "extended_max",
        },
    ];
    assert_eq!(RULE_ELEVATION, 193u16);
    assert_eq!(RULE_RANGE_SLIDER, 250u16);
    assert_eq!(HIGH_RULES[0].kind, 256u16);
    assert_eq!(HIGH_RULES[1].kind, 512u16);
    assert_eq!(HIGH_RULES[2].kind, 65535u16);

    // Ensure a Node carrying >=256 RelativeRule entries renders cleanly without panicking.
    static NODE: Node = Node {
        width: Size::Dp(120.0),
        height: Size::Dp(48.0),
        relative_rules: HIGH_RULES,
        ..android_textappearance::LAYOUT
    };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let commands = ui.frame(Input::default(), bounds, 0.0, |f| {
        NODE.render(f, bounds, &mut state);
    });
    assert!(!commands.is_empty());
}
