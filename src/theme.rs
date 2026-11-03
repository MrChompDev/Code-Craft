// Theme: IntelliJ-inspired dark palette.
use eframe::egui;
use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Vec2, Visuals};

pub const EDITOR_BG: Color32 = Color32::from_rgb(0x1E, 0x1F, 0x22);
pub const PANEL_BG: Color32 = Color32::from_rgb(0x2B, 0x2D, 0x30);
pub const BAR_BG: Color32 = Color32::from_rgb(0x2B, 0x2D, 0x30);
pub const HOVER: Color32 = Color32::from_rgb(0x39, 0x3B, 0x40);
pub const SEL_BG: Color32 = Color32::from_rgb(0x43, 0x45, 0x4A);
pub const SEL_BLUE: Color32 = Color32::from_rgb(0x2E, 0x43, 0x6E);
pub const ACCENT: Color32 = Color32::from_rgb(0x35, 0x74, 0xF0);
pub const RUN_GREEN: Color32 = Color32::from_rgb(0x57, 0x96, 0x5C);
pub const RUN_GREEN_HI: Color32 = Color32::from_rgb(0x63, 0xA4, 0x68);
pub const PLAY_GREEN: Color32 = Color32::from_rgb(0x5F, 0xB8, 0x65);

pub const TEXT: Color32 = Color32::from_rgb(0xDF, 0xE1, 0xE5);
pub const TEXT2: Color32 = Color32::from_rgb(0x9D, 0xA0, 0xA8);
pub const TEXT3: Color32 = Color32::from_rgb(0x6F, 0x73, 0x7A);
pub const BORDER: Color32 = Color32::from_rgb(0x39, 0x3B, 0x40);
pub const BORDER2: Color32 = Color32::from_rgb(0x43, 0x45, 0x4A);

pub const ACTIVE_LINE: Color32 = Color32::from_rgb(0x28, 0x2A, 0x2E);
pub const GUTTER_NUM: Color32 = Color32::from_rgb(0x4B, 0x50, 0x59);
pub const GUTTER_NUM_CUR: Color32 = Color32::from_rgb(0x9D, 0xA0, 0xA8);
pub const FOLD_DOTS: Color32 = Color32::from_rgb(0x6F, 0x73, 0x7A);

// Syntax colors (new-UI dark)
pub const SYN_KW: Color32 = Color32::from_rgb(0xCF, 0x8E, 0x6D);
pub const SYN_STR: Color32 = Color32::from_rgb(0x6A, 0xAB, 0x73);
pub const SYN_NUM: Color32 = Color32::from_rgb(0x2A, 0xAC, 0xB8);
pub const SYN_COM: Color32 = Color32::from_rgb(0x7A, 0x7E, 0x85);
pub const SYN_FN: Color32 = Color32::from_rgb(0x56, 0xA8, 0xF5);
pub const SYN_TYPE: Color32 = Color32::from_rgb(0xC7, 0x7D, 0xBB);
pub const SYN_FIELD: Color32 = Color32::from_rgb(0x98, 0x76, 0xAA);
pub const SYN_PLAIN: Color32 = Color32::from_rgb(0xBC, 0xBE, 0xC4);
pub const SYN_TAG: Color32 = Color32::from_rgb(0xCF, 0x8E, 0x6D);
pub const SYN_ATTR: Color32 = Color32::from_rgb(0xC7, 0x7D, 0xBB);
pub const SYN_SEL: Color32 = Color32::from_rgb(0xE8, 0xA3, 0x5B);
pub const SYN_HEAD: Color32 = Color32::from_rgb(0xCF, 0x8E, 0x6D);

// Log levels
pub const LOG_INFO: Color32 = Color32::from_rgb(0x54, 0x8A, 0xF7);
pub const LOG_WARN: Color32 = Color32::from_rgb(0xE8, 0xA3, 0x3D);
pub const LOG_ERR: Color32 = Color32::from_rgb(0xF7, 0x54, 0x64);
pub const LOG_OK: Color32 = Color32::from_rgb(0x5F, 0xB8, 0x65);

// File-type icon colors
pub const IC_JS: Color32 = Color32::from_rgb(0xE8, 0xBF, 0x6A);
pub const IC_JSON: Color32 = Color32::from_rgb(0xCB, 0xCB, 0x41);
pub const IC_PY: Color32 = Color32::from_rgb(0x4E, 0x8F, 0xE3);
pub const IC_HTML: Color32 = Color32::from_rgb(0xE0, 0x82, 0x3D);
pub const IC_CSS: Color32 = Color32::from_rgb(0x3D, 0x9B, 0xE9);
pub const IC_MD: Color32 = Color32::from_rgb(0x7A, 0x8A, 0xA0);
pub const IC_TXT: Color32 = Color32::from_rgb(0x9D, 0xA0, 0xA8);
pub const IC_RS: Color32 = Color32::from_rgb(0xCE, 0x6F, 0x3F);
pub const FOLDER: Color32 = Color32::from_rgb(0x6F, 0x9B, 0xCE);

pub fn apply(ctx: &egui::Context) {
    ctx.style_mut(|s| {
        let v: &mut Visuals = &mut s.visuals;
        v.dark_mode = true;
        v.override_text_color = Some(TEXT);
        v.window_fill = PANEL_BG;
        v.panel_fill = PANEL_BG;
        v.extreme_bg_color = EDITOR_BG;
        v.faint_bg_color = Color32::from_rgb(0x33, 0x35, 0x39);
        v.selection.bg_fill = SEL_BLUE;
        v.selection.stroke = Stroke::NONE;
        v.hyperlink_color = ACCENT;
        v.window_stroke = Stroke::new(1.0, BORDER2);
        v.window_corner_radius = CornerRadius::same(6);
        v.popup_shadow = Default::default();
        v.window_shadow = Default::default();
        let r = CornerRadius::same(3);
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.corner_radius = r;
        v.widgets.noninteractive.bg_fill = PANEL_BG;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT2);
        v.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        v.widgets.inactive.bg_stroke = Stroke::NONE;
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT2);
        v.widgets.hovered.bg_fill = HOVER;
        v.widgets.hovered.bg_stroke = Stroke::NONE;
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
        v.widgets.active.bg_fill = HOVER;
        v.widgets.active.bg_stroke = Stroke::NONE;
        v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
        s.spacing.item_spacing = Vec2::new(6.0, 3.0);
        s.spacing.button_padding = Vec2::new(8.0, 3.0);
        s.spacing.menu_margin = Margin::same(5);
        s.spacing.icon_width_inner = 12.0;
        s.spacing.icon_width = 14.0;
        s.text_styles = [
            (TextStyle::Body, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(10.5, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(16.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace)),
        ]
        .into();
    });
}

pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}
