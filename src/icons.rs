// Hand-drawn vector icons (no external assets).
use crate::project::Ft;
use crate::theme;
use eframe::egui;
use egui::FontFamily;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

fn seg(p: &Painter, a: Pos2, b: Pos2, col: Color32, w: f32) {
    p.line_segment([a, b], Stroke::new(w, col));
}

pub enum Chev {
    Down,
    Right,
    Up,
}

pub fn chevron(p: &Painter, c: Pos2, s: f32, col: Color32, dir: Chev) {
    let w = 1.4;
    match dir {
        Chev::Right => {
            seg(
                p,
                c + Vec2::new(-s * 0.3, -s * 0.5),
                c + Vec2::new(s * 0.35, 0.0),
                col,
                w,
            );
            seg(
                p,
                c + Vec2::new(s * 0.35, 0.0),
                c + Vec2::new(-s * 0.3, s * 0.5),
                col,
                w,
            );
        }
        Chev::Down => {
            seg(
                p,
                c + Vec2::new(-s * 0.5, -s * 0.3),
                c + Vec2::new(0.0, s * 0.35),
                col,
                w,
            );
            seg(
                p,
                c + Vec2::new(0.0, s * 0.35),
                c + Vec2::new(s * 0.5, -s * 0.3),
                col,
                w,
            );
        }
        Chev::Up => {
            seg(
                p,
                c + Vec2::new(-s * 0.5, s * 0.3),
                c + Vec2::new(0.0, -s * 0.35),
                col,
                w,
            );
            seg(
                p,
                c + Vec2::new(0.0, -s * 0.35),
                c + Vec2::new(s * 0.5, s * 0.3),
                col,
                w,
            );
        }
    }
}

pub fn chevrons_in(p: &Painter, c: Pos2, col: Color32) {
    // two small chevrons pointing at each other (collapse all)
    let s = 3.2;
    seg(
        p,
        c + Vec2::new(-5.5, -s),
        c + Vec2::new(-2.5, 0.0),
        col,
        1.3,
    );
    seg(
        p,
        c + Vec2::new(-2.5, 0.0),
        c + Vec2::new(-5.5, s),
        col,
        1.3,
    );
    seg(p, c + Vec2::new(5.5, -s), c + Vec2::new(2.5, 0.0), col, 1.3);
    seg(p, c + Vec2::new(2.5, 0.0), c + Vec2::new(5.5, s), col, 1.3);
}

pub fn folder(p: &Painter, c: Pos2, col: Color32, open: bool) {
    let w = 1.4;
    let (w2, h) = if open { (7.0, 5.0) } else { (6.5, 4.5) };
    let top = c.y - 3.5;
    let tl = Pos2::new(c.x - w2, top + 1.5);
    seg(p, Pos2::new(tl.x, top), Pos2::new(tl.x + 4.0, top), col, w);
    seg(
        p,
        Pos2::new(tl.x + 4.0, top),
        Pos2::new(tl.x + 5.2, top + 1.5),
        col,
        w,
    );
    seg(
        p,
        Pos2::new(tl.x + 5.2, top + 1.5),
        Pos2::new(c.x + w2, top + 1.5),
        col,
        w,
    );
    seg(
        p,
        Pos2::new(c.x + w2, top + 1.5),
        Pos2::new(c.x + w2, top + 1.5 + h),
        col,
        w,
    );
    seg(
        p,
        Pos2::new(c.x + w2, top + 1.5 + h),
        Pos2::new(tl.x, top + 1.5 + h),
        col,
        w,
    );
    seg(
        p,
        Pos2::new(tl.x, top + 1.5 + h),
        Pos2::new(tl.x, top + 1.5),
        col,
        w,
    );
}

pub fn badge(p: &Painter, c: Pos2, ft: Ft, selected: bool) {
    let (col, label) = match ft {
        Ft::Js => (theme::IC_JS, "JS"),
        Ft::Json => (theme::IC_JSON, "{}"),
        Ft::Py => (theme::IC_PY, "PY"),
        Ft::Html => (theme::IC_HTML, "<>"),
        Ft::Css => (theme::IC_CSS, "#"),
        Ft::Md => (theme::IC_MD, "MD"),
        Ft::Txt => (theme::IC_TXT, "TX"),
        Ft::Rs => (theme::IC_RS, "RS"),
        Ft::Other => (theme::IC_TXT, ""),
    };
    let rect = Rect::from_center_size(c, Vec2::new(13.0, 11.5));
    p.rect_filled(
        rect,
        2.0,
        col.gamma_multiply(if selected { 1.0 } else { 0.92 }),
    );
    if !label.is_empty() {
        p.text(
            c,
            Align2::CENTER_CENTER,
            label,
            FontId::new(6.8, FontFamily::Proportional),
            Color32::from_rgb(0x1E, 0x1F, 0x22),
        );
    } else {
        for i in 0..3 {
            seg(
                p,
                Pos2::new(c.x - 3.0, c.y - 2.5 + i as f32 * 2.5),
                Pos2::new(c.x + 3.0, c.y - 2.5 + i as f32 * 2.5),
                Color32::from_rgb(0x1E, 0x1F, 0x22),
                1.1,
            );
        }
    }
}

pub fn play(p: &Painter, c: Pos2, s: f32, col: Color32) {
    let pts = [
        Pos2::new(c.x - s * 0.42, c.y - s * 0.62),
        Pos2::new(c.x - s * 0.42, c.y + s * 0.62),
        Pos2::new(c.x + s * 0.62, c.y),
    ];
    p.add(egui::Shape::convex_polygon(pts.to_vec(), col, Stroke::NONE));
}

pub fn stop(p: &Painter, c: Pos2, col: Color32) {
    p.rect_filled(Rect::from_center_size(c, Vec2::splat(7.5)), 1.5, col);
}

pub fn bug(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.2;
    // body
    p.circle_stroke(c + Vec2::new(0.0, 0.8), 3.0, Stroke::new(w, col));
    seg(
        p,
        c + Vec2::new(0.0, -3.4),
        c + Vec2::new(0.0, -1.8),
        col,
        w,
    );
    // antennae
    seg(
        p,
        c + Vec2::new(0.0, -1.8),
        c + Vec2::new(-2.2, -3.6),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(0.0, -1.8),
        c + Vec2::new(2.2, -3.6),
        col,
        w,
    );
    // legs
    for dy in [-0.2, 1.2, 2.6] {
        seg(
            p,
            c + Vec2::new(-3.0, 0.8 + dy),
            c + Vec2::new(-5.0, 0.2 + dy),
            col,
            w,
        );
        seg(
            p,
            c + Vec2::new(3.0, 0.8 + dy),
            c + Vec2::new(5.0, 0.2 + dy),
            col,
            w,
        );
    }
}

pub fn gear(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.3;
    p.circle_stroke(c, 3.2, Stroke::new(w, col));
    p.circle_filled(c, 0.8, col);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let d1 = 4.2;
        let d2 = 5.6;
        seg(
            p,
            c + Vec2::new(a.cos() * d1, a.sin() * d1),
            c + Vec2::new(a.cos() * d2, a.sin() * d2),
            col,
            w,
        );
    }
}

pub fn magnifier(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.4;
    p.circle_stroke(c + Vec2::new(-0.8, -0.8), 3.4, Stroke::new(w, col));
    seg(
        p,
        c + Vec2::new(1.8, 1.8),
        c + Vec2::new(4.6, 4.6),
        col,
        w * 1.2,
    );
}

pub fn bell(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.3;
    // dome
    let pts: Vec<Pos2> = (0..=12)
        .map(|i| {
            let t = std::f32::consts::PI * (i as f32 / 12.0);
            Pos2::new(c.x - 4.0 * t.cos(), c.y + 1.0 - 4.6 * t.sin())
        })
        .collect();
    p.add(egui::Shape::line(pts, Stroke::new(w, col)));
    seg(p, c + Vec2::new(-4.6, 1.4), c + Vec2::new(4.6, 1.4), col, w);
    seg(p, c + Vec2::new(-1.6, 3.4), c + Vec2::new(1.6, 3.4), col, w);
    p.circle_filled(c + Vec2::new(0.0, -4.4), 1.0, col);
}

pub fn branch(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.3;
    p.circle_filled(c + Vec2::new(-3.2, -3.4), 1.8, col);
    p.circle_filled(c + Vec2::new(-3.2, 3.4), 1.8, col);
    p.circle_filled(c + Vec2::new(3.2, -3.4), 1.8, col);
    seg(
        p,
        c + Vec2::new(-3.2, -1.6),
        c + Vec2::new(-3.2, 1.6),
        col,
        w,
    );
    seg(p, c + Vec2::new(-3.2, 0.0), c + Vec2::new(2.0, 0.0), col, w);
    seg(
        p,
        c + Vec2::new(3.2, -1.6),
        c + Vec2::new(3.2, -0.6),
        col,
        w,
    );
    seg(p, c + Vec2::new(3.2, -0.6), c + Vec2::new(2.0, 0.0), col, w);
}

pub fn plus(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.5;
    seg(p, c + Vec2::new(-3.4, 0.0), c + Vec2::new(3.4, 0.0), col, w);
    seg(p, c + Vec2::new(0.0, -3.4), c + Vec2::new(0.0, 3.4), col, w);
}

pub fn folder_plus(p: &Painter, c: Pos2, col: Color32) {
    folder(p, c + Vec2::new(-1.2, 0.8), col, false);
    seg(
        p,
        c + Vec2::new(1.6, -3.6),
        c + Vec2::new(1.6, 0.2),
        col,
        1.3,
    );
    seg(
        p,
        c + Vec2::new(-0.4, -1.7),
        c + Vec2::new(3.6, -1.7),
        col,
        1.3,
    );
}

pub fn terminal_ico(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.4;
    seg(
        p,
        c + Vec2::new(-4.0, -2.6),
        c + Vec2::new(-1.4, 0.0),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(-1.4, 0.0),
        c + Vec2::new(-4.0, 2.6),
        col,
        w,
    );
    seg(p, c + Vec2::new(0.4, 2.8), c + Vec2::new(4.2, 2.8), col, w);
}

pub fn person(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.3_f32;
    p.circle_stroke(c + Vec2::new(0.0, -2.4), 2.2, Stroke::new(w, col));
    let pts: Vec<Pos2> = (0..=10)
        .map(|i| {
            let t = std::f32::consts::PI * (i as f32 / 10.0);
            Pos2::new(c.x + 4.4 * t.cos(), c.y + 4.4 - 3.4 * t.sin())
        })
        .collect();
    p.add(egui::Shape::line(pts, Stroke::new(w, col)));
}

pub fn trash(p: &Painter, c: Pos2, col: Color32) {
    let w = 1.2;
    seg(
        p,
        c + Vec2::new(-4.0, -3.0),
        c + Vec2::new(4.0, -3.0),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(-1.8, -3.0),
        c + Vec2::new(-1.8, -4.2),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(1.8, -3.0),
        c + Vec2::new(1.8, -4.2),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(-1.8, -4.2),
        c + Vec2::new(1.8, -4.2),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(-3.2, -3.0),
        c + Vec2::new(-2.6, 4.2),
        col,
        w,
    );
    seg(p, c + Vec2::new(3.2, -3.0), c + Vec2::new(2.6, 4.2), col, w);
    seg(p, c + Vec2::new(-2.6, 4.2), c + Vec2::new(2.6, 4.2), col, w);
    seg(
        p,
        c + Vec2::new(-0.9, -1.4),
        c + Vec2::new(-0.9, 2.6),
        col,
        w,
    );
    seg(p, c + Vec2::new(0.9, -1.4), c + Vec2::new(0.9, 2.6), col, w);
}

pub fn grid_dots(p: &Painter, c: Pos2, col: Color32) {
    for dy in [-2.8, 2.8] {
        for dx in [-2.8, 2.8] {
            p.circle_filled(c + Vec2::new(dx, dy), 1.25, col);
        }
    }
}

pub fn logo(p: &Painter, c: Pos2, col: Color32) {
    // simple mark: two inward chevrons (echoes the personal logo)
    let w = 2.2;
    seg(
        p,
        c + Vec2::new(-7.0, -5.0),
        c + Vec2::new(-2.5, 0.0),
        col,
        w,
    );
    seg(
        p,
        c + Vec2::new(-2.5, 0.0),
        c + Vec2::new(-7.0, 5.0),
        col,
        w,
    );
    seg(p, c + Vec2::new(7.0, -5.0), c + Vec2::new(2.5, 0.0), col, w);
    seg(p, c + Vec2::new(2.5, 0.0), c + Vec2::new(7.0, 5.0), col, w);
}
