use nyashkair::dsp::*;

fn run(freq: f32, amp: f32, mid: f32, high: f32, power: bool) -> (f32, f32) {
    let fs = 48000.0;
    let mut a = Air::new(fs);
    let p = Params { mid, high, trim_db: 0.0, power };
    let (mut ei, mut eo) = (0.0f64, 0.0f64);
    for n in 0..48000 {
        let s = amp * (2.0 * std::f32::consts::PI * freq * n as f32 / fs).sin();
        let mut x = [s, s];
        a.process(&p, &mut x, 2);
        assert!(x[0].is_finite() && x[1].is_finite());
        if n > 24000 {
            ei += (s * s) as f64;
            eo += (x[0] * x[0]) as f64;
        }
    }
    (ei as f32, eo as f32)
}

#[test]
fn off_is_transparent() {
    let (i, o) = run(3000.0, 0.5, 0.0, 0.0, true);
    assert!((o / i - 1.0).abs() < 1e-3, "{}", o / i);
    let (i, o) = run(12000.0, 0.5, 1.0, 1.0, false);
    assert!((o / i - 1.0).abs() < 1e-3);
}

#[test]
fn lows_untouched_highs_lifted() {
    let (i, o) = run(400.0, 0.05, 1.0, 1.0, true);
    let db = 10.0 * (o / i).log10();
    assert!(db.abs() < 0.5, "low band changed by {db} dB");
    let (i, o) = run(12000.0, 0.02, 0.0, 1.0, true);
    let db = 10.0 * (o / i).log10();
    assert!(db > 5.0, "high air lift only {db} dB");
    let (i, o) = run(4500.0, 0.02, 1.0, 0.0, true);
    let db = 10.0 * (o / i).log10();
    assert!(db > 4.0, "mid air lift only {db} dB");
}

#[test]
fn loud_material_gets_less_boost() {
    let (i, o) = run(12000.0, 0.02, 0.0, 1.0, true);
    let quiet = 10.0 * (o / i).log10();
    let (i, o) = run(12000.0, 0.8, 0.0, 1.0, true);
    let loud = 10.0 * (o / i).log10();
    assert!(quiet > loud + 3.0, "quiet {quiet} loud {loud}");
}

#[test]
fn survives_full_scale_square() {
    let fs = 48000.0;
    let mut a = Air::new(fs);
    let p = Params { mid: 1.0, high: 1.0, trim_db: 12.0, power: true };
    for n in 0..20000 {
        let s = if (n / 40) % 2 == 0 { 1.0 } else { -1.0 };
        let mut x = [s, -s];
        a.process(&p, &mut x, 2);
        assert!(x[0].is_finite() && x[0].abs() < 40.0);
    }
}
