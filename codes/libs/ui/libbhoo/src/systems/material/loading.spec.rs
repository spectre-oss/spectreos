use super::*;

#[test]
fn cadence_has_seven_shapes_and_rotation_is_continuous_at_spring_completion() {
    let a = loading_sample(0.0);
    assert_eq!(a.shape, 0);
    assert_eq!(a.progress, 0.0);
    assert_eq!(loading_sample(0.4).shape, 1);
    assert_eq!(loading_sample(0.650).shape, 1);
    assert_eq!(loading_sample(4.550).shape, 0);
    assert!(spring_progress(0.2) > 0.7);
}

#[test]
fn matched_paths_are_closed_finite_and_bounded_through_every_morph() {
    for shape in 0..8 {
        for step in 0..=20 {
            let sample = LoadingSample {
                shape,
                progress: step as f32 / 20.0,
                rotation: step as f32 * 18.0,
            };
            let mut first = None;
            let mut end = None;
            let mut count = 0;
            loading_cubics(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 48.0,
                    height: 48.0,
                },
                sample,
                |c| {
                    assert!(c.iter().all(|v| v.is_finite()));
                    for i in 0..=16 {
                        if i < 16 {
                            let p = [
                                at(&c, i as f32 / 16.0, 0) - 24.0,
                                at(&c, i as f32 / 16.0, 1) - 24.0,
                            ];
                            let q = [
                                at(&c, (i + 1) as f32 / 16.0, 0) - 24.0,
                                at(&c, (i + 1) as f32 / 16.0, 1) - 24.0,
                            ];
                            assert!(
                                p[0] * q[1] - p[1] * q[0] >= -0.001,
                                "Native triangle fan must remain star-shaped"
                            );
                        }
                        let x = at(&c, i as f32 / 16.0, 0);
                        let y = at(&c, i as f32 / 16.0, 1);
                        assert!((-1.0..=49.0).contains(&x) && (-1.0..=49.0).contains(&y));
                    }
                    if first.is_none() {
                        first = Some([c[0], c[1]]);
                    }
                    if let Some(old) = end {
                        let old: [f32; 2] = old;
                        assert!((old[0] - c[0]).abs() < 0.001 && (old[1] - c[1]).abs() < 0.001);
                    }
                    end = Some([c[6], c[7]]);
                    count += 1;
                },
            );
            assert_eq!(count, loading_curve_count(shape));
            let a = first.unwrap();
            let b = end.unwrap();
            assert!((a[0] - b[0]).abs() < 0.001 && (a[1] - b[1]).abs() < 0.001);
        }
    }
}

#[test]
fn pausing_excludes_hidden_time() {
    let mut state = LoadingState::default();
    state.advance(10.0);
    state.advance(10.2);
    state.pause();
    state.advance(100.0);
    assert!((state.elapsed - 0.2).abs() < 1e-8);
}
