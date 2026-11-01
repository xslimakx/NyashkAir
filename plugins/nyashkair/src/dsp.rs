//! Air: dynamic presence + brilliance with harmonic excitation, in parallel with the dry signal.
use nyashk_common::dsp::*;

// Presence band = first-order HP x first-order LP. Both have a non-negative real part, so adding
// the band to the dry signal can only lift, never dip.
pub const MID_HP: f32 = 3500.0;
pub const MID_LP: f32 = 9000.0;
pub const MID_HARM_HP: f32 = 4200.0;
pub const HIGH_HP: f32 = 9000.0; // first order: its real part is never negative, so the boost can't dip
pub const HIGH_HARM_HP: f32 = 11000.0;

pub struct Coefs {
    pub mid_hp: Coef,
    pub mid_lp: Coef,
    pub mid_h: Coef,
    pub high_hp: Coef,
    pub high_h: [Coef; 2],
}

impl Coefs {
    pub fn new(fs: f32) -> Self {
        let q = butter_q::<2>();
        Self {
            mid_hp: Coef::highpass1(fs, MID_HP),
            mid_lp: Coef::lowpass1(fs, MID_LP),
            mid_h: Coef::highpass(fs, MID_HARM_HP, 0.7071),
            high_hp: Coef::highpass1(fs, HIGH_HP),
            high_h: [Coef::highpass(fs, HIGH_HARM_HP, q[0]), Coef::highpass(fs, HIGH_HARM_HP, q[1])],
        }
    }
}

#[derive(Clone, Default)]
struct Ch {
    mid_hp: Bq,
    mid_lp: Bq,
    mid_hh: Bq,
    high_hp: Bq,
    high_hh: Chain<2>,
}

pub struct Params {
    pub mid: f32,  // 0..1
    pub high: f32, // 0..1
    pub trim_db: f32,
    pub power: bool,
}

pub struct Air {
    c: Coefs,
    ch: [Ch; 2],
    os_m: [Os2; 2],
    os_h: [Os2; 2],
    env_m: Env,
    env_h: Env,
    att: f32,
    rel: f32,
    pub in_peak: f32,
    pub out_peak: f32,
    pub added_db: f32,
    sm_mid: Smooth,
    sm_high: Smooth,
    sm_pow: Smooth,
    added: f32,
    dry: f32,
}

impl Air {
    pub fn new(fs: f32) -> Self {
        Self {
            c: Coefs::new(fs),
            ch: Default::default(),
            os_m: [Os2::new(fs), Os2::new(fs)],
            os_h: [Os2::new(fs), Os2::new(fs)],
            env_m: Env::default(),
            env_h: Env::default(),
            att: time_coef(2.0, fs),
            rel: time_coef(90.0, fs),
            in_peak: 0.0,
            out_peak: 0.0,
            added_db: -90.0,
            sm_mid: Smooth::new(0.0, fs, 15.0),
            sm_high: Smooth::new(0.0, fs, 15.0),
            sm_pow: Smooth::new(1.0, fs, 8.0),
            added: 0.0,
            dry: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.ch = Default::default();
        for o in self.os_m.iter_mut().chain(self.os_h.iter_mut()) {
            o.reset();
        }
        self.env_m = Env::default();
        self.env_h = Env::default();
    }

    /// Process one stereo (or mono: n = 1) frame in place.
    pub fn process(&mut self, p: &Params, x: &mut [f32; 2], n: usize) {
        let mid = self.sm_mid.tick(p.mid);
        let high = self.sm_high.tick(p.high);
        let pw = self.sm_pow.tick(if p.power { 1.0 } else { 0.0 });
        let trim = db_to_lin(p.trim_db);

        let mut mid_bp = [0.0f32; 2];
        let mut high_hp = [0.0f32; 2];
        let mut mid_h = [0.0f32; 2];
        let mut high_h = [0.0f32; 2];
        let mut in_pk = 0.0f32;
        for i in 0..n {
            let s = x[i];
            in_pk = in_pk.max(s.abs());
            let c = &self.c;
            let st = &mut self.ch[i];
            let b = st.mid_lp.tick(&c.mid_lp, st.mid_hp.tick(&c.mid_hp, s));
            mid_bp[i] = b;
            // harmonics of the presence band (even + odd), then keep only the new highs
            let m = self.os_m[i].process(b * 3.0, |v| sat_asym(v, 0.25));
            mid_h[i] = st.mid_hh.tick(&c.mid_h, m) * 0.33;
            let h = st.high_hp.tick(&c.high_hp, s);
            high_hp[i] = h;
            let hh = self.os_h[i].process(h * 4.0, |v| sat_asym(v, 0.2));
            high_h[i] = st.high_hh.tick(&c.high_h, hh) * 0.25;
        }
        // shared (linked) dynamics so the stereo image stays put
        let (mut em, mut eh) = (0.0f32, 0.0f32);
        for i in 0..n {
            em = em.max(mid_bp[i].abs());
            eh = eh.max(high_hp[i].abs());
        }
        let dm = 1.0 / (1.0 + 9.0 * self.env_m.tick(em, self.att, self.rel));
        let dh = 1.0 / (1.0 + 9.0 * self.env_h.tick(eh, self.att, self.rel));

        let mut add_e = 0.0f32;
        let mut dry_e = 0.0f32;
        let mut out_pk = 0.0f32;
        for i in 0..n {
            let air_m = (mid_bp[i] * 0.9 + mid_h[i]) * dm * mid * 1.6;
            let air_h = (high_hp[i] * 0.8 + high_h[i]) * dh * high * 3.4;
            let air = (air_m + air_h) * pw;
            add_e += air * air;
            dry_e += mid_bp[i] * mid_bp[i] + high_hp[i] * high_hp[i]; // the highs that are already there
            let y = (x[i] + air) * trim;
            x[i] = y;
            out_pk = out_pk.max(y.abs());
        }
        // meters (slow peak decay is handled by the UI)
        self.in_peak = in_pk;
        self.out_peak = out_pk;
        self.added = self.added * 0.995 + add_e * 0.005;
        self.dry = self.dry * 0.995 + dry_e * 0.005;
        self.added_db = if self.dry < 1e-9 { -90.0 } else { 10.0 * ((self.added + 1e-12) / (self.dry + 1e-12)).log10() };
    }
}

/// Static (linear) part of the air curve for the display: |1 + a_m * BP(f) + a_h * HP(f)| in dB.
pub fn curve_db(fs: f32, mid: f32, high: f32, f: f32) -> f32 {
    let c = Coefs::new(fs);
    let (h0r, h0i) = c.mid_hp.response(fs, f);
    let (l0r, l0i) = c.mid_lp.response(fs, f);
    let (bre, bim) = (h0r * l0r - h0i * l0i, h0r * l0i + h0i * l0r);
    let (xr, xi) = c.high_hp.response(fs, f);
    let (mh_r, _mh_i) = c.mid_h.response(fs, f);
    let mh_mag = (mh_r * mh_r + _mh_i * _mh_i).sqrt();
    // harmonic contribution is not linear; approximate it in the picture by its share of the band
    let mr = mid * 1.6 * 0.9;
    let hrr = high * 3.4 * 0.8;
    let re = 1.0 + mr * bre + hrr * xr + mid * 0.35 * mh_mag * bre.abs();
    let im = mr * bim + hrr * xi;
    10.0 * (re * re + im * im).max(1e-9).log10()
}
