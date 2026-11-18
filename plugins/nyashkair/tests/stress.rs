use nyashkair::dsp::*;

#[test]
fn every_sample_rate_and_extreme_setting_stays_finite_and_bounded() {
    for &fs in &[8000.0f32, 22050.0, 44100.0, 48000.0, 96000.0, 192000.0] {
        for &(mid, high, trim) in &[(0.0, 0.0, 0.0), (1.0, 1.0, 12.0), (1.0, 0.0, -60.0), (0.0, 1.0, 0.0)] {
            let mut a = Air::new(fs);
            let p = Params { mid, high, trim_db: trim, power: true };
            let mut rng = 1u32;
            let mut peak = 0.0f32;
            for n in 0..(fs as usize) {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let w = (rng >> 8) as f32 / 8388608.0 - 1.0;
                let s = match (n / 4000) % 4 {
                    0 => w,
                    1 => 0.0,
                    2 => if (n / 20) % 2 == 0 { 1.0 } else { -1.0 },
                    _ => 1.0, // DC
                };
                let mut x = [s, -s];
                a.process(&p, &mut x, 2);
                assert!(x[0].is_finite() && x[1].is_finite(), "fs {fs} n {n}");
                peak = peak.max(x[0].abs()).max(x[1].abs());
            }
            assert!(peak < 20.0 * 10f32.powf(trim.max(0.0) / 20.0), "fs {fs}: peak {peak}");
        }
    }
}

#[test]
fn recovers_after_nan_or_inf_from_the_host() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut a = Air::new(48000.0);
        let p = Params { mid: 1.0, high: 1.0, trim_db: 0.0, power: true };
        for n in 0..96000 {
            let v = if (100..110).contains(&n) { bad } else { 0.3 * (n as f32 * 0.05).sin() };
            let mut x = [v, v];
            a.process(&p, &mut x, 2);
            if n > 48000 {
                assert!(x[0].is_finite() && x[1].is_finite(), "air {bad}: still broken at n {n}: {x:?}");
            }
        }
    }
}
