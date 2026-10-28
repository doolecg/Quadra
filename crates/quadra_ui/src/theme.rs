// Colours and sizes from mockup A (docs/mockups/a-familiar.html). Sizes in this
// crate are the mockup's CSS pixels; ZOOM scales the whole UI to ~85% of them,
// on top of whatever Windows display scaling is set.

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

pub const ZOOM: f32 = 0.85;

pub const UI: Color32 = Color32::from_rgb(0x44, 0x44, 0x44);
pub const STRIP: Color32 = Color32::from_rgb(0x2f, 0x2f, 0x2f);
pub const INK: Color32 = Color32::from_rgb(0xdc, 0xdc, 0xdc);
pub const DIM: Color32 = Color32::from_rgb(0x8c, 0x8c, 0x8c);
pub const ON: Color32 = Color32::from_rgb(0x5f, 0x8a, 0xc1);
pub const ON_EDGE: Color32 = Color32::from_rgb(0x8f, 0xb2, 0xe0);
pub const VIEWPORT: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
pub const ACTIVE_VIEW: Color32 = Color32::from_rgb(0xa5, 0x8c, 0x3c);
pub const FIELD: Color32 = Color32::from_rgb(0x59, 0x59, 0x59);
pub const FIELD2: Color32 = Color32::from_rgb(0x64, 0x64, 0x64);
pub const BUTTON: Color32 = Color32::from_rgb(0x65, 0x65, 0x65);
pub const ROLLOUT: Color32 = Color32::from_rgb(0x51, 0x51, 0x51);
pub const ROW1: Color32 = Color32::from_rgb(0x4f, 0x4f, 0x4f);
pub const ROW2: Color32 = Color32::from_rgb(0x5a, 0x5a, 0x5a);
pub const HEADER: Color32 = Color32::from_rgb(0x4a, 0x4a, 0x4a);
pub const TAB: Color32 = Color32::from_rgb(0x3b, 0x3b, 0x3b);
pub const TRACK: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub const SEP: Color32 = Color32::from_rgb(0x5a, 0x5a, 0x5a);
pub const KEY_CURSOR: Color32 = Color32::from_rgb(0x8a, 0x7d, 0x2a);
pub const LAYER_TEXT: Color32 = Color32::from_rgb(0xe8, 0xd8, 0x4a);

pub fn apply(ctx: &egui::Context) {
    ctx.set_zoom_factor(ZOOM);

    // Segoe UI is what the mockup uses. It ships with Windows; on other systems
    // egui's bundled font is used instead.
    let mut fonts = FontDefinitions::default();
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts.font_data.insert("segoe".into(), FontData::from_owned(bytes).into());
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "segoe".into());
    }
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, FontId::proportional(11.0)),
            (TextStyle::Body, FontId::proportional(13.0)),
            (TextStyle::Button, FontId::proportional(13.0)),
            (TextStyle::Heading, FontId::proportional(15.0)),
            (TextStyle::Monospace, FontId::monospace(13.0)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.spacing.interact_size.y = 21.0;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = Some(INK);
        v.panel_fill = UI;
        v.window_fill = UI;
        v.extreme_bg_color = FIELD;
        v.faint_bg_color = ROW1;
        v.selection.bg_fill = ON;
        v.selection.stroke.color = INK;
        v.window_corner_radius = 2.into();
        v.menu_corner_radius = 2.into();
        for w in [&mut v.widgets.inactive, &mut v.widgets.noninteractive] {
            w.weak_bg_fill = BUTTON;
            w.bg_fill = BUTTON;
            w.corner_radius = 2.into();
            w.bg_stroke = egui::Stroke::NONE;
        }
        v.widgets.noninteractive.weak_bg_fill = UI;
        v.widgets.noninteractive.bg_fill = UI;
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x74, 0x74, 0x74);
        v.widgets.hovered.bg_fill = Color32::from_rgb(0x74, 0x74, 0x74);
        v.widgets.hovered.corner_radius = 2.into();
        v.widgets.active.weak_bg_fill = ON;
        v.widgets.active.bg_fill = ON;
        v.widgets.active.corner_radius = 2.into();
        v.widgets.open.weak_bg_fill = FIELD2;
    });
}
