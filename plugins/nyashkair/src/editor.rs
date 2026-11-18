//! NyashkAir GUI: modern analog faceplate (pre-rendered in tools/render_ui.py, see hw.rs).
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, epaint::PathStroke, Align2, Color32, FontId, Pos2, Stroke},
};
use nyashk_common::analyzer::{draw_spectrum, Spectrum};
use nyashk_common::ui::{fill_under, resize_grip, set_b, set_norm};
use std::sync::Arc;
use std::time::Duration;

use crate::hw::{self, Hw};
use crate::hw_layout as lay;
use crate::{dsp, AirParams, Meters};

const BASE_W: f32 = lay::W;
const BASE_H: f32 = lay::H;

struct Gui {
    hw: Option<Hw>,
    spec_in: Spectrum,
    spec_out: Spectrum,
    in_pk: f32,
    out_pk: f32,
}

fn k(id: &str) -> &'static hw::KnobL {
    hw::find(lay::KNOBS, id, |x| x.id)
}
fn b(id: &str) -> &'static hw::BtnL {
    hw::find(lay::BUTTONS, id, |x| x.id)
}

pub fn create(params: Arc<AirParams>, meters: Arc<Meters>) -> Option<Box<dyn Editor>> {
    let editor_state = params.editor_state.clone();
    let size_state = editor_state.clone();
    create_egui_editor(
        editor_state,
        Gui { hw: None, spec_in: Spectrum::new(4096), spec_out: Spectrum::new(4096), in_pk: -90.0, out_pk: -90.0 },
        |ctx, st| {
            st.hw = None;
            let mut v = egui::Visuals::light();
            v.override_text_color = Some(Color32::from_rgb(30, 34, 40));
            ctx.set_visuals(v);
        },
        move |ctx, setter, st| {
            ctx.request_repaint_after(Duration::from_millis(33));
            if st.hw.is_none() {
                st.hw = Some(Hw::load(ctx, lay::ASSETS));
            }
            let p = &*params;
            let fall = |cur: f32, target: f32| if target > cur { target } else { cur - 1.2 };
            st.in_pk = fall(st.in_pk, Meters::get(&meters.in_db));
            st.out_pk = fall(st.out_pk, Meters::get(&meters.out_db));
            st.spec_in.update(&meters.tap_in);
            st.spec_out.update(&meters.tap_out);
            let accent = hw::c32(lay::ACCENT);

            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::from_rgb(14, 14, 15))).show(ctx, |ui| {
                let full = ui.max_rect();
                st.hw.as_mut().unwrap().o = full.min;
                let hwr = st.hw.as_ref().unwrap();
                let painter = ui.painter().clone();
                hwr.background(&painter, BASE_W, BASE_H);
                let on = p.power.value();
                if hw::button(ui, hwr, b("power"), on, "Switch the effect on or off (smooth, click-free).") {
                    set_b(setter, &p.power, !on);
                }

                // ---- curve over the live spectrum
                let scr = hw::find(lay::SCREENS, "curve", |x| x.id);
                let g = hwr.rect(scr.rect).shrink(6.0);
                let (f_lo, f_hi) = (1000.0f32, 20000.0f32);
                let (d_lo, d_hi) = (-3.0f32, 16.0f32);
                let fx = |f: f32| g.left() + g.width() * (f / f_lo).ln() / (f_hi / f_lo).ln();
                let dy = |d: f32| g.bottom() - g.height() * ((d - d_lo) / (d_hi - d_lo)).clamp(0.0, 1.0);
                let grid = Color32::from_rgb(24, 30, 40);
                let dim = Color32::from_rgb(90, 104, 124);
                for d in [0.0f32, 5.0, 10.0, 15.0] {
                    painter.hline(g.left()..=g.right(), dy(d), Stroke::new(1.0_f32, if d == 0.0 { Color32::from_rgb(44, 54, 70) } else { grid }));
                    painter.text(Pos2::new(g.left() + 3.0, dy(d) - 6.0), Align2::LEFT_CENTER, if d.abs() < 0.5 { "0".to_string() } else { format!("{:+.0}", d) }, FontId::monospace(8.5), dim);
                }
                for f in [2000.0f32, 5000.0, 10000.0] {
                    painter.vline(fx(f), g.top()..=g.bottom(), Stroke::new(1.0_f32, grid));
                    painter.text(Pos2::new(fx(f) + 3.0, g.bottom() - 7.0), Align2::LEFT_CENTER, format!("{}k", f / 1000.0), FontId::monospace(8.5), dim);
                }
                let fs = f32::from_bits(meters.fs.load(std::sync::atomic::Ordering::Relaxed)).clamp(8000.0, 384000.0);
                let sy = |d: f32| g.bottom() - g.height() * ((d + 84.0) / 84.0).clamp(-0.05, 1.05);
                let f_top = f_hi.min(fs * 0.49);
                let out_pts = st.spec_out.points(fs, f_lo, f_top, 140, 4.5, fx, sy);
                draw_spectrum(&painter, g, &out_pts, Color32::from_rgb(20, 28, 42), Color32::from_rgb(64, 82, 110));
                let in_pts = st.spec_in.points(fs, f_lo, f_top, 140, 4.5, fx, sy);
                painter.with_clip_rect(g).add(egui::Shape::line(in_pts, PathStroke::new(1.0_f32, Color32::from_rgb(70, 76, 86))));
                let (m, h) = (p.mid.value() / 100.0, p.high.value() / 100.0);
                let pts: Vec<Pos2> = (0..=160)
                    .map(|i| {
                        let f = f_lo * (f_hi / f_lo).powf(i as f32 / 160.0);
                        let d = if on { dsp::curve_db(48000.0, m, h, f) } else { 0.0 };
                        Pos2::new(fx(f), dy(d))
                    })
                    .collect();
                fill_under(&painter.with_clip_rect(g), &pts, dy(0.0), Color32::from_rgba_unmultiplied(60, 160, 255, 40));
                painter.with_clip_rect(g).add(egui::Shape::line(pts, PathStroke::new(2.0_f32, accent)));
                hw::glass(&painter, hwr, scr);

                let lv = |d: f32| ((d + 48.0) / 48.0).clamp(0.0, 1.0);
                hw::ladder(&painter, hwr, hw::find(lay::LADDERS, "in", |x| x.id), lv(st.in_pk), false);
                hw::ladder(&painter, hwr, hw::find(lay::LADDERS, "out", |x| x.id), lv(st.out_pk), false);
                let added = Meters::get(&meters.added_db);
                hw::text(&painter, hwr, hw::find(lay::TEXTS, "added", |x| x.id), &if on && added > -60.0 { format!("{:+.1} dB", added) } else { "-- dB".into() });

                // ---- knobs
                let link = p.link.value();
                let a = hw::knob(ui, hwr, setter, &p.mid, k("mid"), accent, "MID AIR: presence, 2 to 8 kHz. A gentle lift plus fresh harmonics, reduced automatically on loud passages.");
                let bb = hw::knob(ui, hwr, setter, &p.high, k("high"), accent, "HIGH AIR: brilliance above 8 kHz. A silky lift plus new high harmonics.");
                if link {
                    if a != 0.0 {
                        set_norm(setter, &p.high, p.high.unmodulated_normalized_value() + a);
                    } else if bb != 0.0 {
                        set_norm(setter, &p.mid, p.mid.unmodulated_normalized_value() + bb);
                    }
                }
                if hw::button(ui, hwr, b("link"), link, "LINK: moving one knob moves the other by the same amount.") {
                    set_b(setter, &p.link, !link);
                }
                hw::knob(ui, hwr, setter, &p.trim, k("trim"), accent, "TRIM: output trim in dB.");
                resize_grip(ui, ctx, &size_state, full, BASE_W, BASE_H, 480.0);
            });
        },
    )
}
