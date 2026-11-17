//! Deeper behaviour checks of the air / presence exciter.
use nyashkair::dsp::{curve_db, Air, Params};
use std::f32::consts::PI;

fn tone_gain(fs: f32, f: f32, amp: f32, mid: f32, high: f32) -> f32 {
    let mut a = Air::new(fs);
    let p = Params { mid, high, trim_db: 0.0, power: true };
    let n = (fs * 0.6) as usize;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    let (mut ri, mut ii) = (0.0f64, 0.0f64);
    for i in 0..n {
        let s = amp * (2.0 * PI * f * i as f32 / fs).sin();
        let mut x = [s, s];
        a.process(&p, &mut x, 2);
        if i > n / 3 {
            let ph = 2.0 * std::f64::consts::PI * f as f64 * i as f64 / fs as f64;
            re += x[0] as f64 * ph.cos();
            im += x[0] as f64 * ph.sin();
            ri += s as f64 * ph.cos();
            ii += s as f64 * ph.sin();
        }
    }
    10.0 * ((re * re + im * im) / (ri * ri + ii * ii)).log10() as f32
}

fn spectrum_at(y: &[f32], fs: f32, f: f32) -> f32 {
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (i, v) in y.iter().enumerate() {
        let w = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / y.len() as f64).cos();
        let ph = 2.0 * std::f64::consts::PI * f as f64 * i as f64 / fs as f64;
        re += *v as f64 * w * ph.cos();
        im += *v as f64 * w * ph.sin();
    }
    20.0 * ((re * re + im * im).sqrt() * 4.0 / y.len() as f64).max(1e-12).log10() as f32
}

#[test]
fn knobs_lift_their_own_region_monotonically() {
    let mut last_m = -1.0;
    let mut last_h = -1.0;
    for k in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let gm = tone_gain(48000.0, 5000.0, 0.01, k, 0.0);
        let gh = tone_gain(48000.0, 14000.0, 0.01, 0.0, k);
        println!("knob {k:.2}: mid @5k {gm:+.2} dB, high @14k {gh:+.2} dB");
        assert!(gm >= last_m - 0.01 && gh >= last_h - 0.01, "not monotonic");
        last_m = gm;
        last_h = gh;
    }
    assert!(last_m > 3.0 && last_h > 3.0, "full knobs should lift clearly");
    // the other region barely moves
    let low = tone_gain(48000.0, 300.0, 0.01, 1.0, 1.0);
    println!("300 Hz at full knobs: {low:+.2} dB");
    assert!(low.abs() < 0.5, "lows touched: {low}");
}

#[test]
fn display_curve_matches_the_sound_at_low_level() {
    for fs in [44100.0f32, 48000.0, 96000.0] {
        for (m, h) in [(0.5f32, 0.0f32), (0.0, 0.6), (0.4, 0.4)] {
            for f in [2000.0f32, 5000.0, 8000.0, 12000.0, 16000.0] {
                let meas = tone_gain(fs, f, 0.003, m, h);
                let shown = curve_db(fs, m, h, f);
                assert!((meas - shown).abs() < 1.5, "fs {fs} mid {m} high {h} at {f}: sound {meas:.2}, graph {shown:.2}");
            }
        }
    }
}

#[test]
fn loud_highs_get_less_boost() {
    let q = tone_gain(48000.0, 10000.0, 0.003, 0.0, 0.8);
    let l = tone_gain(48000.0, 10000.0, 0.3, 0.0, 0.8);
    println!("10 kHz boost: quiet {q:.2} dB, loud {l:.2} dB");
    assert!(l < q - 1.0, "no level dependence");
    assert!(l > 0.0, "loud highs must not be cut");
}

#[test]
fn excitation_without_audible_aliasing() {
    // a loud 15 kHz tone at full high air: harmonics fold back in a plain saturator (the 3rd at
    // 45 kHz would land on 3 kHz at 48 kHz); the oversampled stage must keep that inaudible
    let fs = 48000.0;
    let mut a = Air::new(fs);
    let p = Params { mid: 1.0, high: 1.0, trim_db: 0.0, power: true };
    let mut y = vec![];
    for i in 0..48000 {
        let s = 0.5 * (2.0 * PI * 15000.0 * i as f32 / fs).sin();
        let mut x = [s, s];
        a.process(&p, &mut x, 2);
        if i >= 16384 {
            y.push(x[0]);
        }
    }
    let fund = spectrum_at(&y, fs, 15000.0);
    let alias = [3000.0f32, 18000.0, 6000.0].iter().map(|f| spectrum_at(&y, fs, *f)).fold(-200.0f32, f32::max);
    println!("15 kHz {fund:.1} dB, worst alias {alias:.1} dB");
    assert!(alias < fund - 60.0, "aliasing {alias} vs {fund}");
}

#[test]
fn trim_and_power() {
    let g = |trim: f32, power: bool| {
        let mut a = Air::new(48000.0);
        let p = Params { mid: 0.8, high: 0.8, trim_db: trim, power };
        let mut pk = 0.0f32;
        for i in 0..48000 {
            let s = 0.1 * (2.0 * PI * 200.0 * i as f32 / 48000.0).sin();
            let mut x = [s, s];
            a.process(&p, &mut x, 2);
            if i > 24000 {
                pk = pk.max(x[0].abs());
            }
        }
        20.0 * (pk / 0.1).log10()
    };
    assert!((g(-6.0, true) + 6.0).abs() < 0.2);
    assert!((g(6.0, true) - 6.0).abs() < 0.2);
}
