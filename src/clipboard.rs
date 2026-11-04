//! Edit > Cut, Copy and Paste of views and models, through the system clipboard.
//!
//! As in StarUML (src/engine/clipboard-manager.js, default-commands.js:614-774), the clipboard holds
//! JSON text, so a copy can be pasted in another window. The JSON layout is UMLCraft's own: StarUML's
//! exact layout could not be checked, so the two programs do not paste into each other.
//!
//! - Views copied from the canvas paste as new views of the same models, centred on the screen.
//! - A model copied from the explorer pastes as a deep copy with new ids, into the selected element.

use crate::app::{App, Area};
use crate::model::{num, Eid};
use crate::views::{self, is_edge, P, R};
use serde_json::{json, Value};
use std::collections::HashSet;

const MARK: &str = "umlcraft-clipboard";

/// What the clipboard holds, once parsed.
pub enum Clip {
    Views { diagram_type: String, views: Vec<Value>, models: Vec<String> },
    Model { field: String, model: Value },
}

pub fn parse(text: &str) -> Option<Clip> {
    let v: Value = serde_json::from_str(text).ok()?;
    if v.get("format")?.as_str()? != MARK {
        return None;
    }
    match v.get("kind")?.as_str()? {
        "views" => Some(Clip::Views {
            diagram_type: v.get("diagramType")?.as_str()?.to_string(),
            views: v.get("views")?.as_array()?.clone(),
            models: v.get("models")?.as_array()?.iter().filter_map(|m| m.as_str().map(String::from)).collect(),
        }),
        "model" => Some(Clip::Model { field: v.get("field")?.as_str()?.to_string(), model: v.get("model")?.clone() }),
        _ => None,
    }
}

/// Shift every box and line in exported views by (dx, dy).
fn shift(v: &mut Value, dx: f64, dy: f64) {
    match v {
        Value::Object(m) => {
            for (k, d) in [("left", dx), ("top", dy)] {
                if let Some(x) = m.get(k).and_then(|x| x.as_f64()) {
                    m.insert(k.into(), num(x + d));
                }
            }
            if let Some(Value::String(pts)) = m.get("points") {
                let moved: Vec<P> = views::parse_points(pts).into_iter().map(|p| P { x: p.x + dx, y: p.y + dy }).collect();
                m.insert("points".into(), json!(views::format_points(&moved)));
            }
            m.values_mut().for_each(|x| shift(x, dx, dy));
        }
        Value::Array(a) => a.iter_mut().for_each(|x| shift(x, dx, dy)),
        _ => {}
    }
}

/// Drop references to views that are not part of the copy (containedViews, containerView).
fn trim_view_refs(v: &mut Value, keep: &HashSet<String>) {
    if let Value::Object(m) = v {
        if let Some(Value::Array(a)) = m.get_mut("containedViews") {
            a.retain(|r| r.get("$ref").and_then(|x| x.as_str()).map(|id| keep.contains(id)).unwrap_or(false));
            if a.is_empty() {
                m.remove("containedViews");
            }
        }
        if let Some(id) = m.get("containerView").and_then(|r| r.get("$ref")).and_then(|x| x.as_str()) {
            if !keep.contains(id) {
                m.remove("containerView");
            }
        }
        m.values_mut().for_each(|x| trim_view_refs(x, keep));
    } else if let Value::Array(a) = v {
        a.iter_mut().for_each(|x| trim_view_refs(x, keep));
    }
}

impl App {
    /// The model a copy, cut or paste in the explorer acts on: the focused element, unless the canvas
    /// was used last and has a selection.
    fn model_target(&self) -> Option<Eid> {
        if self.last_area == Area::Canvas && !self.selection.is_empty() {
            return None;
        }
        self.focus.filter(|f| self.repo.alive(*f) && self.repo.ty(*f) != "Project" && !self.is_view(*f))
    }

    /// The selected views that can be copied: boxes, and lines whose two ends are copied too.
    fn copyable_views(&self) -> Vec<Eid> {
        let Some(d) = self.diagram else { return vec![] };
        let top: Vec<Eid> = self.repo.list(d, "ownedViews").into_iter().filter(|v| self.selection.contains(v)).collect();
        let nodes: HashSet<Eid> = top.iter().copied().filter(|v| !is_edge(&self.repo, *v)).collect();
        top.into_iter()
            .filter(|v| {
                !is_edge(&self.repo, *v)
                    || [self.repo.refd(*v, "tail"), self.repo.refd(*v, "head")].iter().all(|e| e.map(|e| nodes.contains(&e)).unwrap_or(false))
            })
            .collect()
    }

    pub fn can_copy(&self) -> bool {
        self.model_target().is_some() || !self.copyable_views().is_empty()
    }

    /// Edit > Copy: the clipboard text for the selection, or None (default-commands.js:638-674).
    pub fn copy_text(&mut self) -> Option<String> {
        if let Some(m) = self.model_target() {
            let field = self.repo.field_of(m)?;
            return Some(json!({ "format": MARK, "kind": "model", "field": field, "model": self.repo.export(m) }).to_string());
        }
        let vs = self.copyable_views();
        if vs.is_empty() {
            if !self.selection.is_empty() {
                self.message = Some(("Copy".into(), "No views to copy or some of selected views cannot be copied.".into()));
            }
            return None;
        }
        let d = self.diagram?;
        let keep: HashSet<String> = vs.iter().map(|v| self.repo.id(*v)).collect();
        let mut models = vec![];
        let mut out = vec![];
        for v in &vs {
            let mut all = vec![];
            self.repo.descendants(*v, &mut all);
            for x in all {
                if let Some(m) = self.repo.refd(x, "model") {
                    let id = self.repo.id(m);
                    if !models.contains(&id) {
                        models.push(id);
                    }
                }
            }
            let mut e = self.repo.export(*v);
            trim_view_refs(&mut e, &keep);
            out.push(e);
        }
        Some(json!({ "format": MARK, "kind": "views", "diagramType": self.repo.ty(d), "views": out, "models": models }).to_string())
    }

    /// Put text on the system clipboard (and keep it for a paste from the menu).
    pub fn set_clipboard(&mut self, ctx: Option<&egui::Context>, text: String) {
        if let Some(ctx) = ctx {
            ctx.copy_text(text.clone());
        }
        self.clipboard = Some(text);
    }

    /// The clipboard text, read from the system clipboard when possible.
    pub fn get_clipboard(&mut self) -> Option<String> {
        self.system_clipboard().and_then(|c| c.get_text().ok()).or_else(|| self.clipboard.clone())
    }

    fn system_clipboard(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.sys_clipboard.is_none() {
            self.sys_clipboard = arboard::Clipboard::new().ok();
        }
        self.sys_clipboard.as_mut()
    }

    /// Edit > Copy Diagram As Image (Ctrl+Shift+C): deselect everything, draw the whole diagram the way
    /// PNG export does and put the picture on the system clipboard. The original copies a PNG on macOS
    /// and a JPEG elsewhere, so outside macOS the picture gets a white background (default-commands.js:678-700).
    pub fn copy_diagram_image(&mut self, png: Result<Vec<u8>, String>) {
        self.select(vec![]);
        let img = png.and_then(|p| image::load_from_memory(&p).map_err(|e| e.to_string())).map(|i| i.to_rgba8());
        let mut img = match img {
            Ok(i) => i,
            Err(e) => {
                self.message = Some(("Failed to copy the diagram.".into(), e));
                return;
            }
        };
        if !cfg!(target_os = "macos") {
            for p in img.pixels_mut() {
                let a = p.0[3] as u32;
                for c in 0..3 {
                    p.0[c] = ((p.0[c] as u32 * a + 255 * (255 - a)) / 255) as u8;
                }
                p.0[3] = 255;
            }
        }
        let data = arboard::ImageData { width: img.width() as usize, height: img.height() as usize, bytes: std::borrow::Cow::Owned(img.into_raw()) };
        let ok = self.system_clipboard().map(|c| c.set_image(data).is_ok()).unwrap_or(false);
        if !ok {
            self.message = Some(("Failed to copy the diagram.".into(), "The system clipboard is not available.".into()));
        }
    }

    pub fn cmd_copy(&mut self, ctx: Option<&egui::Context>) {
        if let Some(t) = self.copy_text() {
            self.set_clipboard(ctx, t);
        }
    }

    /// Edit > Cut: copy, then delete the model (explorer) or remove the views (canvas).
    pub fn cmd_cut(&mut self, ctx: Option<&egui::Context>) {
        if let Some(m) = self.model_target() {
            if let Some(t) = self.copy_text() {
                self.set_clipboard(ctx, t);
                self.change(|a| a.delete_models(&[m]));
                self.focus = None;
            }
            return;
        }
        let vs = self.copyable_views();
        if vs.is_empty() || vs.len() != self.selection.len() {
            self.message = Some(("Cut".into(), "No views to cut or some of selected views cannot be cut.".into()));
            return;
        }
        if let Some(t) = self.copy_text() {
            self.set_clipboard(ctx, t);
            self.change(|a| {
                a.remove_views(&vs);
                a.selection.clear();
            });
        }
    }

    /// Whether `target` can take an element of type `ty` in `field`.
    pub fn can_hold(&self, target: Eid, ty: &str, field: &str) -> bool {
        let t = self.repo.ty(target);
        let pkg = self.repo.kind_of(target, "UMLPackage") || t == "Project";
        match field {
            "attributes" | "operations" => self.can_add_feature(target, ty),
            "literals" => t == "UMLEnumeration",
            "parameters" => t == "UMLOperation",
            "ownedElements" if t == "Project" => crate::meta::is_kind_of(ty, "UMLPackage") || crate::meta::is_kind_of(ty, "Diagram") || ty.ends_with("Diagram"),
            "ownedElements" => {
                pkg || (self.repo.kind_of(target, "UMLClassifier") && (crate::meta::is_kind_of(ty, "UMLClassifier") || ty.ends_with("Diagram") || crate::meta::is_kind_of(ty, "Relationship")))
            }
            _ => false,
        }
    }

    /// Whether an element may be moved under `target` in the explorer: into the same-named field, which
    /// the target must be able to hold, and never into itself or something inside it (engine.js:585-614).
    pub fn can_relocate(&self, src: Eid, target: Eid) -> bool {
        if !self.repo.alive(src) || !self.repo.alive(target) || src == target || self.repo.parent(src) == Some(target) || self.is_view(target) {
            return false;
        }
        if self.repo.path(target).contains(&src) {
            return false;
        }
        let Some(field) = self.repo.field_of(src) else { return false };
        self.can_hold(target, self.repo.ty(src), &field)
    }

    /// Edit > Paste of the given clipboard text (default-commands.js:702-774).
    pub fn paste_text(&mut self, text: &str) {
        match parse(text) {
            Some(Clip::Model { field, model }) => self.paste_model(&field, model),
            Some(Clip::Views { diagram_type, views, models }) => self.paste_views(&diagram_type, views, &models),
            None => {}
        }
    }

    fn paste_model(&mut self, field: &str, model: Value) {
        let ty = model.get("_type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let Some(sel) = self.focus.filter(|f| self.repo.alive(*f)) else { return };
        let sel = if self.is_view(sel) { self.repo.refd(sel, "model").unwrap_or(sel) } else { sel };
        // into the selected element, or else into its owner (so copy + paste on one node duplicates it)
        let target = [Some(sel), self.repo.parent(sel)].into_iter().flatten().find(|t| self.can_hold(*t, &ty, field));
        let Some(target) = target else {
            self.toast("The element in clipboard cannot be pasted here.");
            return;
        };
        let new = self.change(|a| a.repo.insert_copies(vec![model], target, field));
        if let Some(&n) = new.first() {
            self.expanded.insert(target);
            self.focus = Some(n);
            self.selection.clear();
            self.reveal(n);
        }
    }

    fn paste_views(&mut self, diagram_type: &str, mut views: Vec<Value>, models: &[String]) {
        let Some(d) = self.diagram else { return };
        // every model must still exist, otherwise nothing happens
        if !models.iter().all(|id| self.repo.by_id.get(id).map(|e| self.repo.alive(*e)).unwrap_or(false)) {
            return;
        }
        if self.repo.ty(d) != diagram_type {
            self.message = Some(("Paste".into(), "Views in clipboard cannot be pasted in this diagram.".into()));
            return;
        }
        // the pasted group is centred on the screen centre
        // boxes decide the group's extent; lines follow them (their end points are laid out again)
        let has_box = views.iter().any(|v| v.get("points").is_none());
        let mut b: Option<R> = None;
        for v in &views {
            if has_box && v.get("points").is_some() {
                continue;
            }
            let r = if let Some(pts) = v.get("points").and_then(|p| p.as_str()) {
                let pts = views::parse_points(pts);
                let (x1, y1) = pts.iter().fold((f64::INFINITY, f64::INFINITY), |a, p| (a.0.min(p.x), a.1.min(p.y)));
                let (x2, y2) = pts.iter().fold((f64::NEG_INFINITY, f64::NEG_INFINITY), |a, p| (a.0.max(p.x), a.1.max(p.y)));
                if pts.is_empty() {
                    continue;
                }
                R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
            } else {
                let g = |k: &str| v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0);
                R { x: g("left"), y: g("top"), w: g("width"), h: g("height") }
            };
            b = Some(match b {
                None => r,
                Some(o) => {
                    let (x, y) = (o.x.min(r.x), o.y.min(r.y));
                    R { x, y, w: (o.x + o.w).max(r.x + r.w) - x, h: (o.y + o.h).max(r.y + r.h) - y }
                }
            });
        }
        let centre = match self.canvas_rect {
            Some(rect) => P { x: self.scroll.x + rect.width() as f64 / self.zoom / 2.0, y: self.scroll.y + rect.height() as f64 / self.zoom / 2.0 },
            None => b.map(|b| P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 }).unwrap_or_default(),
        };
        if let Some(b) = b {
            let dx = (centre.x - (b.x + b.w / 2.0)).round().max(-b.x);
            let dy = (centre.y - (b.y + b.h / 2.0)).round().max(-b.y);
            views.iter_mut().for_each(|v| shift(v, dx, dy));
        }
        let new = self.change(|a| a.repo.insert_copies(views, d, "ownedViews"));
        self.select(new);
    }

    pub fn cmd_paste(&mut self) {
        if let Some(t) = self.get_clipboard() {
            self.paste_text(&t);
        }
    }
}
