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
