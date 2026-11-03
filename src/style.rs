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

pub fn hex(c: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}

impl App {
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
