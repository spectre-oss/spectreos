use bhoo::{TextMetrics,TextStyle};
#[test]
fn width_uses_one_advance_per_grapheme_and_spacing_between_clusters(){
    let mut metrics=TextMetrics::default();metrics.normal['e' as usize-32]=0.4;
    let style=TextStyle{size:20.,letter_spacing:2.,..TextStyle::default()};
    assert_eq!(metrics.width("e\u{301}",style),metrics.width("e",style));
    assert_eq!(metrics.width("e\u{301}e",style),18.);
    assert_eq!(metrics.width("👩\u{200d}💻",style),20.);
    assert_eq!(metrics.width("👩\u{200d}💻!",style),33.);
    assert_eq!(metrics.width("\u{200d}\u{200b}",style),0.);
    assert_eq!(metrics.width("中",style),20.);
    assert_eq!(metrics.width("é",style),12.);
    assert_eq!(metrics.width("🇫🇷",style),20.);
}
