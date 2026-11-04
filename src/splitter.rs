//! Panel splitters (utils/resizer.js, ui/panel-manager.js): the sidebar, the navigator, the Toolbox,
//! the Editors and the panels under the diagram can be resized by dragging their inner edge.
//!
//! Dragging a collapsible panel below 10 px hides it; a hidden sidebar or navigator keeps a grab
//! handle at the window edge, so dragging that out again shows it at the dragged size. Sizes are
//! kept in the preferences as `_viewState.<panel>.size`.

use crate::app::App;
use egui::{CursorIcon, Id, Rect, Sense};

/// Panels below this many pixels while dragged are hidden (resizer.js:303-323).
pub const COLLAPSE_BELOW: f32 = 10.0;
/// The Toolbox and Editors never get smaller (main-view.html:54, 60).
pub const MIN_SPLIT: f32 = 60.0;
/// The view above the Toolbox or Editors keeps at least this height (sidebar-view.js:18, navigator-view.js:18).
pub const KEEP_ABOVE: f32 = 40.0;
/// The width of the area that catches a splitter drag.
const GRIP: f32 = 6.0;

/// Panels with a splitter, and their default size.
pub const PANELS: [(&str, f32); 9] = [
    ("sidebar", crate::ui::SIDEBAR_WIDTH),
    ("navigator", crate::ui::NAVIGATOR_WIDTH),
    ("toolbox", crate::ui::TOOLBOX_HEIGHT),
    ("editors", crate::ui::EDITORS_HEIGHT),
    ("validation", 160.0),
    ("markdown", 200.0),
    ("thumbnails", 130.0),
    ("relationships", 160.0),
    ("findResults", 160.0),
];

/// Which side of the panel the splitter is on.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Edge {
    Right,
    Left,
    Top,
}

/// What a drag asks for.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Split {
    Resize(f32),
    Hide,
}

/// The size a drag gives, from the size it started at and how far it went: below 10 px a collapsible
/// panel hides; otherwise the size is kept between `min` and `max` (resizer.js:303-353).
pub fn drag_result(start: f32, moved: f32, min: f32, max: f32, collapsible: bool) -> Split {
    let raw = start + moved;
    if collapsible && raw < COLLAPSE_BELOW {
        return Split::Hide;
    }
    Split::Resize(raw.min(max).max(min))
}

impl App {
    /// A panel's size, as last dragged or its default.
    pub fn panel_size(&self, id: &str) -> f32 {
        self.panel_sizes.get(id).copied().or_else(|| PANELS.iter().find(|p| p.0 == id).map(|p| p.1)).unwrap_or(160.0)
    }

    /// A splitter along `edge` of `rect` (the panel's rectangle, or an empty strip at the window edge
    /// for a hidden panel, with `shown` false). Returns what the drag asks for, if anything.
    pub fn splitter(&mut self, ui: &egui::Ui, id: &'static str, rect: Rect, edge: Edge, shown: bool, min: f32, max: f32, collapsible: bool) -> Option<Split> {
        let strip = match edge {
            Edge::Right => Rect::from_x_y_ranges(rect.right() - GRIP / 2.0..=rect.right() + GRIP / 2.0, rect.y_range()),
            Edge::Left => Rect::from_x_y_ranges(rect.left() - GRIP / 2.0..=rect.left() + GRIP / 2.0, rect.y_range()),
            Edge::Top => Rect::from_x_y_ranges(rect.x_range(), rect.top() - GRIP / 2.0..=rect.top() + GRIP / 2.0),
        };
        let r = ui.interact(strip, Id::new(("splitter", id)), Sense::drag());
        if r.hovered() || r.dragged() {
            ui.ctx().set_cursor_icon(if edge == Edge::Top { CursorIcon::ResizeVertical } else { CursorIcon::ResizeHorizontal });
        }
        if r.drag_started() {
            self.split_drag = Some((id, if shown { self.panel_size(id) } else { 0.0 }, 0.0));
        }
        let mut out = None;
        if r.dragged() {
            if let Some((did, start, moved)) = self.split_drag.as_mut().filter(|d| d.0 == id) {
                let _ = did;
                let d = r.drag_delta();
                *moved += match edge {
                    Edge::Right => d.x,
                    Edge::Left => -d.x,
                    Edge::Top => -d.y,
                };
                let res = drag_result(*start, *moved, min, max, collapsible);
                if let Split::Resize(s) = res {
                    self.panel_sizes.insert(id.to_string(), s);
                }
                out = Some(res);
            }
        }
        if r.drag_stopped() {
            self.split_drag = None;
        }
        out
    }
}
