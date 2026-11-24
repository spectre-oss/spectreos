use super::*;

#[test]
fn classic_indeterminate_timeline_stays_bounded_and_repeats() {
    for i in 0..2000 {
        let t = i as f64 / 1000.0;
        let a = classic_linear_segments(t);
        let b = classic_linear_segments(t + 2.0);
        for (segment, repeated) in a.into_iter().zip(b) {
            assert!(segment[0] >= 0.0 && segment[1] <= 1.0 && segment[0] <= segment[1]);
            for (x, y) in segment.into_iter().zip(repeated) {
                assert!((x - y).abs() < 0.0001);
            }
        }
        let (start, sweep) = classic_circular_sample(t);
        assert!(start.is_finite() && sweep > 0.0 && sweep < std::f32::consts::TAU);
    }
}

#[test]
fn two_linear_segments_are_ordered_bounded_and_periodic() {
    for i in 0..1750 {
        let s = linear_segments(i as f64 / 1000.0);
        for [tail, head] in s {
            assert!(tail >= 0.0 && head <= 1.0 && tail <= head);
        }
    }
    assert_eq!(linear_segments(0.0), linear_segments(1.750));
}

#[test]
fn circular_sweep_is_continuous_bounded_and_periodic() {
    let a = circular_sample(0.0);
    assert_eq!(a, circular_sample(6.0));
    for i in 0..6000 {
        let (_, s) = circular_sample(i as f64 / 1000.0);
        assert!((0.62..=5.47).contains(&s));
    }
}
