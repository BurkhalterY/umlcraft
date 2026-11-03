//! View > Relationships (Ctrl+Alt+R): a panel under the diagram listing the relationships of the
//! selected element and the element at the other end of each (extensions/default/relationship-view).

use crate::app::App;
use crate::icons;
use crate::model::Eid;
use egui::{Sense, Vec2};

/// One row: the relationship, the element at the other end, and how that end is named.
#[derive(Clone, Debug, PartialEq)]
pub struct RelRow {
    pub rel: Eid,
    pub other: Eid,
    pub role: String,
}

impl App {
    /// Every relationship of `m`, in project order. The role is "(target)" or "(source)" for a directed
    /// relationship and the opposite end's name for an association (relationship-view/main.js:162).
    pub fn relationship_rows(&self, m: Eid) -> Vec<RelRow> {
        let r = &self.repo;
        let mut out = vec![];
        for rel in r.all().filter(|x| crate::views::edge_view_of(r.ty(*x)).is_some()) {
            let Some((t, h)) = self.rel_ends(rel) else { continue };
            if t != m && h != m {
                continue;
            }
            let undirected = r.has(rel, "end1");
            // a self-relationship lists its target end
            let (other, other_end) = if t == m { (h, "end2") } else { (t, "end1") };
            let role = if undirected {
                r.child(rel, other_end).map(|e| r.name(e)).unwrap_or_default()
            } else if t == m {
                "(target)".to_string()
            } else {
                "(source)".to_string()
            };
            out.push(RelRow { rel, other, role });
        }
        out
    }

    /// The panel: the relationship (icon, name, type) and the related element with its role. A double
    /// click selects the relationship; the context menu selects either one.
    pub fn relationships_ui(&mut self, ui: &mut egui::Ui) {
        let mut close = false;
        let subject = self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f) && self.repo.kind_of(*f, "Model"));
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let title = match subject {
                Some(m) => format!("Relationships — {}", self.label_of(m)),
                None => "Relationships".to_string(),
            };
            ui.label(egui::RichText::new(title).size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("×").clicked() {
                    close = true;
                }
            });
        });
        let rows = subject.map(|m| self.relationship_rows(m)).unwrap_or_default();
        let mut go: Option<Eid> = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("relationships").num_columns(2).spacing([24.0, 2.0]).striped(true).show(ui, |ui| {
                for row in rows {
                    let mut resp = None;
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        icons::draw(ui.painter(), ir, icons::key_for(&self.repo, row.rel));
                        let name = self.repo.name(row.rel);
                        let text = format!("{}{}[{}]", name, if name.is_empty() { "" } else { " " }, self.repo.ty(row.rel).trim_start_matches("UML"));
                        resp = Some(ui.selectable_label(self.focus == Some(row.rel), text));
                    });
                    ui.horizontal(|ui| {
                        let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        icons::draw(ui.painter(), ir, icons::key_for(&self.repo, row.other));
                        ui.label(format!("{}  {}", self.label_of(row.other), row.role));
                    });
                    ui.end_row();
                    if let Some(r) = resp {
                        if r.double_clicked() {
                            go = Some(row.rel);
                        }
                        r.context_menu(|ui| {
                            if ui.button("Select Related Element").clicked() {
                                go = Some(row.other);
                                ui.close();
                            }
                            if ui.button("Select Relationship").clicked() {
                                go = Some(row.rel);
                                ui.close();
                            }
                        });
                    }
                }
            });
        });
        if let Some(e) = go {
            self.focus = Some(e);
            self.reveal(e);
        }
        if close {
            self.show_relationships = false;
        }
    }
}
