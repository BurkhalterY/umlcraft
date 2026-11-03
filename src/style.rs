//! View colours: Format > Fill Color and Line Color, and the colour swatches of the Style editor.
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
