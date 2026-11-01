//! NyashkAir GUI.
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, epaint::PathStroke, Align2, Color32, Pos2, Rect, Stroke, Vec2},
};
use nyashk_common::ui::*;
use std::sync::Arc;
use std::time::Duration;

use crate::{dsp, AirParams, Meters};

const BASE_W: f32 = 640.0;
const BASE_H: f32 = 430.0;

struct Gui {
    in_pk: f32,
    out_pk: f32,
    in_hold: f32,
    out_hold: f32,
}

pub fn create(params: Arc<AirParams>, meters: Arc<Meters>) -> Option<Box<dyn Editor>> {
    let editor_state = params.editor_state.clone();
    let size_state = editor_state.clone();
    create_egui_editor(
        editor_state,
        Gui { in_pk: -90.0, out_pk: -90.0, in_hold: -90.0, out_hold: -90.0 },
        |ctx, _| apply_theme(ctx),
        move |ctx, setter, st| {
            ctx.request_repaint_after(Duration::from_millis(33));
            let p = &*params;
            let fall = |cur: f32, target: f32| if target > cur { target } else { cur - 1.2 };
            st.in_pk = fall(st.in_pk, Meters::get(&meters.in_db));
            st.out_pk = fall(st.out_pk, Meters::get(&meters.out_db));
            st.in_hold = st.in_hold.max(st.in_pk) - 0.12;
            st.out_hold = st.out_hold.max(st.out_pk) - 0.12;

            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(BG)).show(ctx, |ui| {
                let full = ui.max_rect();
                let o = full.min;
                let rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(Pos2::new(o.x + x, o.y + y), Vec2::new(w, h));
                header(ui, o, BASE_W, "Air", "AIR / PRESENCE EXCITER");
                let on = p.power.value();
                if pill_tip(ui, rect(BASE_W - 14.0 - 76.0, 22.0, 76.0, 24.0), "POWER", on, "Switch the effect on or off (smooth, click-free).") {
                    set_b(setter, &p.power, !on);
                }

                // ---- display
                let dp = rect(14.0, 70.0, BASE_W - 28.0, 168.0);
                panel(ui, dp);
                let g = Rect::from_min_max(Pos2::new(dp.left() + 40.0, dp.top() + 14.0), Pos2::new(dp.right() - 130.0, dp.bottom() - 26.0));
                let painter = ui.painter().clone();
                painter.rect_filled(g, 4.0, Color32::from_rgb(0x07, 0x07, 0x07));
                let (f_lo, f_hi) = (1000.0f32, 20000.0f32);
                let (d_lo, d_hi) = (-3.0f32, 16.0f32);
                let fx = |f: f32| g.left() + g.width() * (f / f_lo).ln() / (f_hi / f_lo).ln();
                let dy = |d: f32| g.bottom() - g.height() * ((d - d_lo) / (d_hi - d_lo)).clamp(0.0, 1.0);
                for d in [0.0f32, 5.0, 10.0, 15.0] {
                    painter.hline(g.left()..=g.right(), dy(d), Stroke::new(1.0, if d == 0.0 { MID } else { LINE }));
                    painter.text(Pos2::new(g.left() - 6.0, dy(d)), Align2::RIGHT_CENTER, format!("{:+.0}", d), mono(9.5), DIM);
                }
                for f in [1000.0f32, 2000.0, 5000.0, 10000.0, 20000.0] {
                    painter.vline(fx(f), g.top()..=g.bottom(), Stroke::new(1.0, LINE));
                    painter.text(Pos2::new(fx(f), g.bottom() + 11.0), Align2::CENTER_CENTER, if f >= 1000.0 { format!("{}k", f / 1000.0) } else { format!("{}", f) }, mono(9.5), DIM);
                }
                let (m, h) = (p.mid.value() / 100.0, p.high.value() / 100.0);
                let pts: Vec<Pos2> = (0..=160)
                    .map(|i| {
                        let f = f_lo * (f_hi / f_lo).powf(i as f32 / 160.0);
                        let d = if on { dsp::curve_db(48000.0, m, h, f) } else { 0.0 };
                        Pos2::new(fx(f), dy(d))
                    })
                    .collect();
                let mut fill = pts.clone();
                fill.push(Pos2::new(g.right(), dy(0.0)));
                fill.push(Pos2::new(g.left(), dy(0.0)));
                painter.with_clip_rect(g).add(egui::Shape::convex_polygon(fill, Color32::from_white_alpha(18), Stroke::NONE));
                painter.with_clip_rect(g).add(egui::Shape::line(pts, PathStroke::new(2.0, WHITE)));
                painter.text(Pos2::new(g.left() + 8.0, g.top() + 10.0), Align2::LEFT_CENTER, "AIR CURVE", mono(9.5), DIM);

                // meters
                let mx = dp.right() - 112.0;
                let bar_t = dp.top() + 18.0;
                let bar_h = dp.height() - 52.0;
                for (k, (name, lvl, hold)) in [("IN", st.in_pk, st.in_hold), ("OUT", st.out_pk, st.out_hold)].into_iter().enumerate() {
                    let r = Rect::from_min_size(Pos2::new(mx + 38.0 * k as f32, bar_t), Vec2::new(14.0, bar_h));
                    vbar(ui, r, lvl, hold, -48.0, 0.0);
                    painter.text(Pos2::new(r.center().x, r.bottom() + 11.0), Align2::CENTER_CENTER, name, mono(9.5), DIM);
                }
                let added = Meters::get(&meters.added_db);
                painter.text(Pos2::new(dp.right() - 22.0, bar_t + 8.0), Align2::RIGHT_CENTER, "ADDED", mono(9.0), DIM);
                painter.text(Pos2::new(dp.right() - 22.0, bar_t + 26.0), Align2::RIGHT_CENTER, if on && added > -60.0 { format!("{:+.1} dB", added) } else { "-- dB".into() }, mono(11.0), WHITE);

                // ---- knobs
                let kp = rect(14.0, 250.0, BASE_W - 28.0, 166.0);
                panel(ui, kp);
                let link = p.link.value();
                let cell = |cx: f32| Rect::from_min_size(Pos2::new(o.x + cx - 90.0, kp.top() + 6.0), Vec2::new(180.0, 150.0));
                let a = knob_r(ui, setter, &p.mid, "MID AIR", cell(150.0), "Presence: 2 to 8 kHz. Adds a gentle lift plus fresh harmonics, reduced automatically on loud passages.", false, 46.0);
                let b = knob_r(ui, setter, &p.high, "HIGH AIR", cell(BASE_W - 150.0 - 80.0), "Brilliance above 8 kHz. Adds a silky lift plus new high harmonics (like opening a window on the top end).", false, 46.0);
                if link {
                    if a.delta != 0.0 {
                        set_norm(setter, &p.high, p.high.unmodulated_normalized_value() + a.delta);
                    } else if b.delta != 0.0 {
                        set_norm(setter, &p.mid, p.mid.unmodulated_normalized_value() + b.delta);
                    }
                }
                if pill_tip(ui, Rect::from_center_size(Pos2::new(o.x + (150.0 + BASE_W - 230.0) / 2.0, kp.top() + 54.0), Vec2::new(46.0, 26.0)), "LINK", link, "Link the two knobs: moving one moves the other by the same amount.") {
                    set_b(setter, &p.link, !link);
                }
                knob_r(ui, setter, &p.trim, "TRIM", Rect::from_min_size(Pos2::new(o.x + BASE_W - 14.0 - 110.0, kp.top() + 28.0), Vec2::new(100.0, 120.0)), "Output trim in dB.", true, 26.0);

                resize_grip(ui, ctx, &size_state, full, BASE_W, BASE_H, 480.0);
            });
        },
    )
}
