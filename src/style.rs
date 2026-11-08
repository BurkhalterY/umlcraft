//! View colours and fonts: Format > Font, Fill Color and Line Color, and the Style editor's controls.
//!
//! As in StarUML (default-commands.js:966-991, style-editor-view.js:229-278, 549-592), a colour is
//! merged over the selected views and applied to all of them. Colours are written as `#rrggbb`.

use crate::app::App;
use crate::model::Eid;
use crate::render::{parse_color, Rgba};
use serde_json::json;
use std::time::{Duration, Instant};

/// Colour fields of a view, with the label used in the menus and the Style editor.
pub const COLORS: [(&str, &str); 3] = [("fillColor", "Fill Color"), ("lineColor", "Line Color"), ("fontColor", "Text Color")];

/// `#rrggbb`, or `#rrggbbaa` when the colour is not opaque.
pub fn hex(c: Rgba) -> String {
    if c.3 == 255 {
        format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c.0, c.1, c.2, c.3)
    }
}

/// The Color dialog (dialog-manager.js:484-527): the field it edits, the colour picked so far and
/// the hex text being typed. OK applies the colour to the selected views; Cancel leaves them.
#[derive(Clone)]
pub struct ColorDialog {
    pub field: &'static str,
    pub color: Rgba,
    pub text: String,
}

/// The Color dialog's preset palette: 16 columns, a grey ramp then four shades of 16 hues
/// (estimated: the colours of StarUML's palette are not known).
pub fn palette() -> Vec<Rgba> {
    let mut out: Vec<Rgba> = (0..16).map(|i| {
        let g = 255 - (i * 255 / 15) as u8;
        Rgba(g, g, g, 255)
    }).collect();
    for (s, l) in [(0.9, 0.85), (0.85, 0.65), (0.8, 0.45), (0.8, 0.28)] {
        for i in 0..16 {
            out.push(hsl(i as f64 * 22.5, s, l));
        }
    }
    out
}

fn hsl(h: f64, s: f64, l: f64) -> Rgba {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let (r, g, b) = match (h / 60.0) as i64 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let u = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba(u(r), u(g), u(b), 255)
}

/// What the Color dialog's hex field accepts: `#rrggbb` or `#rrggbbaa`, the `#` optional.
pub fn parse_hex(text: &str) -> Option<Rgba> {
    let t = text.trim();
    let t = t.strip_prefix('#').unwrap_or(t);
    if t.len() != 6 && t.len() != 8 {
        return None;
    }
    let n = |i: usize| t.get(i..i + 2).and_then(|x| u8::from_str_radix(x, 16).ok());
    Some(Rgba(n(0)?, n(2)?, n(4)?, if t.len() == 8 { n(6)? } else { 255 }))
}

impl App {
    /// Open the Color dialog on a field, starting from the merged colour (white when it is mixed).
    pub fn open_color_dialog(&mut self, field: &'static str) {
        if self.color_views(field).is_empty() {
            return;
        }
        let def = if field == "fillColor" { Rgba(255, 255, 255, 255) } else { Rgba(0, 0, 0, 255) };
        let color = self.color_state(field).unwrap_or(def);
        self.color_dialog = Some(ColorDialog { field, color, text: hex(color) });
    }

    /// The selected views that have this colour field.
    pub fn color_views(&self, field: &str) -> Vec<Eid> {
        self.selection.iter().copied().filter(|v| self.repo.alive(*v) && crate::meta::order_of(self.repo.ty(*v), field).is_some()).collect()
    }

    /// The colour every selected view shares, if they agree.
    pub fn color_state(&self, field: &str) -> Option<Rgba> {
        let vs = self.color_views(field);
        let def = if field == "fillColor" { Rgba(255, 255, 255, 255) } else { Rgba(0, 0, 0, 255) };
        let first = parse_color(&self.repo.str(*vs.first()?, field), def);
        vs.iter().all(|v| parse_color(&self.repo.str(*v, field), def) == first).then_some(first)
    }

    /// Apply a colour to every selected view that has the field. Changes made one after the other while
    /// a colour picker is dragged are one undo step.
    pub fn set_color(&mut self, field: &str, c: Rgba) {
        let vs = self.color_views(field);
        if vs.is_empty() {
            return;
        }
        let now = Instant::now();
        let fresh = match &self.style_edit {
            Some((f, t)) => f != field || now.duration_since(*t) > Duration::from_millis(800),
            None => true,
        };
        if fresh {
            self.checkpoint();
        }
        self.style_edit = Some((field.to_string(), now));
        for v in vs {
            self.repo.set(v, field, json!(hex(c)));
        }
        self.done();
    }
}

/// Font sizes offered by the Style editor and the Font dialog (style-editor-view.html).
pub const SIZES: [f64; 14] = [8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 36.0, 42.0];

/// The font faces the Style editor, the Font dialog and Preferences list: the names of the fonts the
/// original loads from resources/fonts (its "default" fallback faces aside), sorted
/// (font-manager.js; style-editor-view.js:795-805; dialog-manager.js:535-541). UMLCraft stores the
/// chosen name; diagrams are drawn with the Arial-compatible face whatever it is.
pub const FONT_FACES: [&str; 12] = ["Arial", "Courier", "Courier New", "Gloria Hallelujah", "Helvetica", "Inter", "Lato", "Roboto", "Source Code Pro", "Source Sans Pro", "Times", "Times New Roman"];

/// The drop-down beside a font face field: picking a face returns it, to copy into the field.
pub fn font_face_menu(ui: &mut egui::Ui, id: &str) -> Option<String> {
    let mut out = None;
    egui::ComboBox::from_id_salt(("font-faces", id)).selected_text("").width(20.0).show_ui(ui, |ui| {
        for f in FONT_FACES {
            if ui.selectable_label(false, f).clicked() {
                out = Some(f.to_string());
            }
        }
    });
    out
}

impl App {
    fn font_views(&self) -> Vec<Eid> {
        self.selection.iter().copied().filter(|v| self.repo.alive(*v) && crate::meta::is_kind_of(self.repo.ty(*v), "View")).collect()
    }

    /// The font face and size every selected view shares, each None when they differ.
    pub fn font_state(&self) -> (Option<String>, Option<f64>) {
        let fonts: Vec<(String, f64)> = self.font_views().into_iter().map(|v| crate::views::view_font(&self.repo, v)).collect();
        let Some(first) = fonts.first().cloned() else { return (None, None) };
        let face = fonts.iter().all(|f| f.0 == first.0).then_some(first.0);
        let size = fonts.iter().all(|f| f.1 == first.1).then_some(first.1);
        (face, size)
    }

    /// Set the face and/or size of every selected view, keeping each one's style (default-commands.js:940-964).
    pub fn set_font(&mut self, face: Option<&str>, size: Option<f64>) {
        let vs = self.font_views();
        if vs.is_empty() || (face.is_none() && size.is_none()) {
            return;
        }
        self.change(|a| {
            for v in vs {
                let (f0, s0) = crate::views::view_font(&a.repo, v);
                let style = a.repo.str(v, "font").split(';').nth(2).and_then(|x| x.parse().ok()).unwrap_or(0);
                let f = face.map(|x| x.to_string()).unwrap_or(f0);
                let sz = size.unwrap_or(s0);
                a.repo.set(v, "font", json!(crate::views::font_str(&f, sz, style)));
            }
        });
    }
}

/// Format > Stereotype Display values, labels and shortcut digits (uml.json, keymaps/uml.json).
pub const STEREOTYPE_DISPLAYS: [(&str, &str); 6] = [
    ("none", "None"),
    ("label", "Label"),
    ("decoration", "Decoration"),
    ("decoration-label", "Decoration with Label"),
    ("icon", "Icon"),
    ("icon-label", "Icon with Label"),
];

impl App {
    /// The stereotype display the selected views share, if they agree.
    pub fn stereotype_display_state(&self) -> Option<String> {
        let vs = self.flag_views("stereotypeDisplay");
        let first = self.repo.str(*vs.first()?, "stereotypeDisplay");
        vs.iter().all(|v| self.repo.str(*v, "stereotypeDisplay") == first).then_some(first)
    }

    /// Set stereotypeDisplay on every selected view that has it (uml-commands.js:266-269, 4565-4595).
    pub fn set_stereotype_display(&mut self, value: &str) {
        let vs = self.flag_views("stereotypeDisplay");
        if vs.is_empty() {
            return;
        }
        self.change(|a| {
            for v in vs {
                a.repo.set(v, "stereotypeDisplay", json!(value));
            }
        });
    }
}
