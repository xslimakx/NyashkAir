//! NyashkAir: dynamic air / presence exciter (VST3).

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub mod dsp;
mod editor;

pub struct Meters {
    pub in_db: AtomicU32,
    pub out_db: AtomicU32,
    pub added_db: AtomicU32,
}
impl Meters {
    fn new() -> Self {
        let m90 = (-90f32).to_bits();
        Self { in_db: AtomicU32::new(m90), out_db: AtomicU32::new(m90), added_db: AtomicU32::new(m90) }
    }
    pub fn get(a: &AtomicU32) -> f32 {
        f32::from_bits(a.load(Ordering::Relaxed))
    }
}

#[derive(Params)]
pub struct AirParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "power"]
    pub power: BoolParam,
    #[id = "mid"]
    pub mid: FloatParam,
    #[id = "high"]
    pub high: FloatParam,
    #[id = "link"]
    pub link: BoolParam,
    #[id = "trim"]
    pub trim: FloatParam,
}

impl Default for AirParams {
    fn default() -> Self {
        let pct = |name: &str, def: f32| {
            FloatParam::new(name, def, FloatRange::Linear { min: 0.0, max: 100.0 }).with_step_size(0.1).with_smoother(SmoothingStyle::Linear(20.0))
        };
        Self {
            editor_state: EguiState::from_size(640, 430),
            power: BoolParam::new("Power", true),
            mid: pct("Mid Air", 20.0),
            high: pct("High Air", 30.0),
            link: BoolParam::new("Link", false),
            trim: FloatParam::new("Trim", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 }).with_unit(" dB").with_step_size(0.1).with_smoother(SmoothingStyle::Linear(20.0)),
        }
    }
}

pub struct NyashkAir {
    params: Arc<AirParams>,
    meters: Arc<Meters>,
    air: Option<dsp::Air>,
}

impl Default for NyashkAir {
    fn default() -> Self {
        Self { params: Arc::new(AirParams::default()), meters: Arc::new(Meters::new()), air: None }
    }
}

impl Plugin for NyashkAir {
    const NAME: &'static str = "NyashkAir";
    const VENDOR: &'static str = "Nyashk";
    const URL: &'static str = "https://github.com/xslimakx";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout { main_input_channels: NonZeroU32::new(2), main_output_channels: NonZeroU32::new(2), ..AudioIOLayout::const_default() },
        AudioIOLayout { main_input_channels: NonZeroU32::new(1), main_output_channels: NonZeroU32::new(1), ..AudioIOLayout::const_default() },
    ];
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _e: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.meters.clone())
    }

    fn initialize(&mut self, _l: &AudioIOLayout, config: &BufferConfig, _c: &mut impl InitContext<Self>) -> bool {
        self.air = Some(dsp::Air::new(config.sample_rate));
        true
    }

    fn reset(&mut self) {
        if let Some(a) = self.air.as_mut() {
            a.reset();
        }
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _c: &mut impl ProcessContext<Self>) -> ProcessStatus {
        nyashk_common::demo::sanitize(buffer);
        let Some(air) = self.air.as_mut() else { return ProcessStatus::Normal };
        let p = &self.params;
        let (mut ipk, mut opk) = (0.0f32, 0.0f32);
        for mut frame in buffer.iter_samples() {
            let dp = dsp::Params {
                mid: p.mid.smoothed.next() / 100.0,
                high: p.high.smoothed.next() / 100.0,
                trim_db: p.trim.smoothed.next(),
                power: p.power.value(),
            };
            let n = frame.len().min(2);
            let mut x = [0.0f32; 2];
            for (i, s) in frame.iter_mut().enumerate().take(n) {
                x[i] = *s;
            }
            air.process(&dp, &mut x, n);
            ipk = ipk.max(air.in_peak);
            opk = opk.max(air.out_peak);
            for (i, s) in frame.iter_mut().enumerate().take(n) {
                *s = x[i];
            }
        }
        let db = |v: f32| 20.0 * (v + 1e-9).log10();
        self.meters.in_db.store(db(ipk).to_bits(), Ordering::Relaxed);
        self.meters.out_db.store(db(opk).to_bits(), Ordering::Relaxed);
        self.meters.added_db.store(air.added_db.to_bits(), Ordering::Relaxed);
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for NyashkAir {
    const VST3_CLASS_ID: [u8; 16] = *b"NyashkAirVST3!!!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Filter];
}

nih_export_vst3!(NyashkAir);
