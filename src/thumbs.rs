//! View > Diagram Thumbnails (Ctrl+Alt+T): a panel under the diagram with a small picture of every
//! diagram in the project (extensions/default/diagram-thumbnails).

use crate::app::App;
use crate::canvas::{EguiMeasure, ScreenCanvas};
use crate::model::Eid;
use crate::views::P;
use egui::{Color32, Sense, Stroke, Vec2};

/// The size of a thumbnail (main.js:85).
pub const THUMB_W: f32 = 120.0;
pub const THUMB_H: f32 = 80.0;

/// How a diagram fits a thumbnail: its bounds grown by 10 on each side (20 in all), scaled to fit.
/// Returns (scale, top-left corner in diagram coordinates).
pub fn thumb_fit(b: Option<crate::views::R>) -> (f64, P) {
    match b {
        Some(b) => {
            let s = (THUMB_W as f64 / (b.w + 20.0)).min(THUMB_H as f64 / (b.h + 20.0));
            (s, P { x: b.x - 10.0, y: b.y - 10.0 })
        }
        None => (1.0, P { x: 0.0, y: 0.0 }),
    }
}

impl App {
    /// "name (from parent)", the thumbnail's label.
    pub fn thumb_label(&self, d: Eid) -> String {
        match self.repo.parent(d).filter(|p| self.repo.ty(*p) != "Project") {
            Some(p) => format!("{} (from {})", self.repo.name(d), self.repo.name(p)),
            None => self.repo.name(d),
        }
    }

    /// Open a diagram from its thumbnail: select it in the explorer, make it current and scroll to (0, 0).
    pub fn open_from_thumb(&mut self, d: Eid) {
        self.focus = Some(d);
        self.reveal(d);
        self.open_diagram(d);
        self.scroll = P { x: 0.0, y: 0.0 };
    }

    /// The panel: one card per diagram in project order. A click selects the diagram; a double click,
    /// or a click on its name, opens it.
    pub fn thumbnails_ui(&mut self, ui: &mut egui::Ui) {
        let mut close = false;
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Diagram Thumbnails").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("×").clicked() {
                    close = true;
                }
            });
        });
        let diagrams = self.print_diagrams(true);
        let ms = EguiMeasure { ctx: ui.ctx().clone() };
        let mut open = None;
        egui::ScrollArea::horizontal().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                for d in diagrams {
                    ui.vertical(|ui| {
                        ui.set_width(THUMB_W);
                        let (r, resp) = ui.allocate_exact_size(Vec2::new(THUMB_W, THUMB_H), Sense::click());
                        let p = ui.painter().with_clip_rect(r);
                        p.rect_filled(r, 0.0, Color32::WHITE);
                        let info = crate::views::layout_diagram(&mut self.repo, &ms, d);
                        let (s, at) = thumb_fit(crate::render::bounds(&self.repo, d));
                        let mut c = ScreenCanvas { painter: &p, origin: r.min, scroll: at, zoom: s, measure: EguiMeasure { ctx: ui.ctx().clone() } };
                        crate::render::draw_diagram(&mut c, &self.repo, d, &info);
                        let current = self.diagram == Some(d);
                        let chosen = self.focus == Some(d);
                        let edge = if current || chosen { crate::canvas::c32(crate::render::SELECT_COLOR) } else { Color32::from_gray(90) };
                        ui.painter().rect_stroke(r, 0.0, Stroke::new(if chosen { 2.0 } else { 1.0 }, edge), egui::StrokeKind::Outside);
                        let name = ui.add(egui::Label::new(egui::RichText::new(self.thumb_label(d)).size(11.0)).truncate().sense(Sense::click()));
                        if resp.clicked() {
                            self.focus = Some(d);
                        }
                        if resp.double_clicked() || name.clicked() {
                            open = Some(d);
                        }
                    });
                    ui.add_space(8.0);
                }
            });
        });
        if let Some(d) = open {
            self.open_from_thumb(d);
        }
        if close {
            self.show_thumbnails = false;
        }
    }
}
