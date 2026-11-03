//! View > Markdown Documentation (Ctrl+Alt+D): a panel under the diagram to edit the selected
//! element's documentation as Markdown, or to preview it (extensions/default/markdown).
//!
//! The preview understands the common Markdown subset: headings, paragraphs, bullet and numbered
//! lists, block quotes, fenced code blocks, horizontal rules, and inline **bold**, *italic* and `code`.

use crate::app::App;
use crate::model::Eid;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Stroke};
use serde_json::json;

/// One block of a Markdown text.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(usize, String),
    Paragraph(String),
    Bullet(usize, String),
    Numbered(usize, String, String),
    Quote(String),
    Code(String),
    Rule,
}

/// Split a Markdown text into blocks.
pub fn parse(text: &str) -> Vec<Block> {
    let mut out = vec![];
    let mut para: Vec<String> = vec![];
    let mut code: Option<Vec<String>> = None;
    let flush = |para: &mut Vec<String>, out: &mut Vec<Block>| {
        if !para.is_empty() {
            out.push(Block::Paragraph(para.join(" ")));
            para.clear();
        }
    };
    for line in text.lines() {
        if let Some(c) = code.as_mut() {
            if line.trim_start().starts_with("```") {
                out.push(Block::Code(c.join("\n")));
                code = None;
            } else {
                c.push(line.to_string());
            }
            continue;
        }
        let t = line.trim();
        let indent = (line.len() - line.trim_start().len()) / 2;
        if t.starts_with("```") {
            flush(&mut para, &mut out);
            code = Some(vec![]);
        } else if t.is_empty() {
            flush(&mut para, &mut out);
        } else if let Some(level) = (1..=6).find(|n| t.starts_with(&format!("{} ", "#".repeat(*n)))) {
            flush(&mut para, &mut out);
            out.push(Block::Heading(level, t[level + 1..].trim().to_string()));
        } else if t.len() >= 3 && (t.chars().all(|c| c == '-') || t.chars().all(|c| c == '*') || t.chars().all(|c| c == '_')) {
            flush(&mut para, &mut out);
            out.push(Block::Rule);
        } else if let Some(rest) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")).or_else(|| t.strip_prefix("+ ")) {
            flush(&mut para, &mut out);
            out.push(Block::Bullet(indent, rest.to_string()));
        } else if let Some((n, rest)) = t.split_once(". ").filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())) {
            flush(&mut para, &mut out);
            out.push(Block::Numbered(indent, n.to_string(), rest.to_string()));
        } else if let Some(rest) = t.strip_prefix('>') {
            flush(&mut para, &mut out);
            out.push(Block::Quote(rest.trim().to_string()));
        } else {
            para.push(t.to_string());
        }
    }
    if let Some(c) = code {
        out.push(Block::Code(c.join("\n")));
    }
    flush(&mut para, &mut out);
    out
}

/// Inline spans: (text, bold, italic, code).
pub fn spans(text: &str) -> Vec<(String, bool, bool, bool)> {
    let mut out = vec![];
    let (mut bold, mut italic) = (false, false);
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let push = |cur: &mut String, out: &mut Vec<(String, bool, bool, bool)>, b: bool, it: bool| {
        if !cur.is_empty() {
            out.push((std::mem::take(cur), b, it, false));
        }
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '`' {
            if let Some(end) = chars[i + 1..].iter().position(|x| *x == '`') {
                push(&mut cur, &mut out, bold, italic);
                out.push((chars[i + 1..i + 1 + end].iter().collect(), false, false, true));
                i += end + 2;
                continue;
            }
        }
        if (c == '*' || c == '_') && chars.get(i + 1) == Some(&c) {
            push(&mut cur, &mut out, bold, italic);
            bold = !bold;
            i += 2;
            continue;
        }
        if c == '*' || (c == '_' && (i == 0 || !chars[i - 1].is_alphanumeric())) || (c == '_' && italic) {
            push(&mut cur, &mut out, bold, italic);
            italic = !italic;
            i += 1;
            continue;
        }
        cur.push(c);
        i += 1;
    }
    push(&mut cur, &mut out, bold, italic);
    out
}

fn inline_job(text: &str, size: f32, color: Color32, strong: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (t, b, it, code) in spans(text) {
        let mut f = TextFormat { font_id: FontId::proportional(size), color, italics: it, ..Default::default() };
        if code {
            f.font_id = FontId::monospace(size - 1.0);
            f.background = Color32::from_white_alpha(18);
        }
        if b || strong {
            // no bold face in the UI font: brighter text stands for it
            f.color = Color32::WHITE;
        }
        job.append(&t, 0.0, f);
    }
    job.wrap.max_width = f32::INFINITY;
    job
}

/// Draw the preview of a Markdown text.
pub fn preview(ui: &mut egui::Ui, text: &str) {
    let color = ui.visuals().text_color();
    for b in parse(text) {
        match b {
            Block::Heading(level, t) => {
                let size = [22.0, 19.0, 16.0, 14.0, 13.0, 12.0][level - 1];
                ui.add_space(4.0);
                ui.label(inline_job(&t, size, color, true));
                if level <= 2 {
                    let r = ui.max_rect();
                    ui.painter().hline(r.x_range(), ui.cursor().top(), Stroke::new(1.0, Color32::from_gray(90)));
                }
            }
            Block::Paragraph(t) => {
                ui.label(inline_job(&t, 13.0, color, false));
            }
            Block::Bullet(indent, t) => {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(12.0 + indent as f32 * 16.0);
                    ui.label("•");
                    ui.label(inline_job(&t, 13.0, color, false));
                });
            }
            Block::Numbered(indent, n, t) => {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(12.0 + indent as f32 * 16.0);
                    ui.label(format!("{n}."));
                    ui.label(inline_job(&t, 13.0, color, false));
                });
            }
            Block::Quote(t) => {
                egui::Frame::NONE.inner_margin(egui::Margin { left: 10, right: 0, top: 2, bottom: 2 }).stroke(Stroke::new(1.0, Color32::from_gray(90))).show(ui, |ui| {
                    ui.label(inline_job(&t, 13.0, Color32::from_gray(170), false));
                });
            }
            Block::Code(t) => {
                egui::Frame::NONE.fill(Color32::from_black_alpha(60)).inner_margin(6).show(ui, |ui| {
                    ui.label(egui::RichText::new(t).monospace());
                });
            }
            Block::Rule => {
                ui.separator();
            }
        }
    }
}

impl App {
    /// The element whose documentation the panel shows: the focused element, when it can have one.
    pub fn markdown_subject(&self) -> Option<Eid> {
        self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f) && self.repo.kind_of(*f, "ExtensibleModel"))
    }

    /// The panel: Edit or Preview (Edit first); the edit is written back on blur as an undoable change.
    pub fn markdown_ui(&mut self, ui: &mut egui::Ui) {
        let subject = self.markdown_subject();
        // follows the selection, and reloads when the text changes elsewhere while not being edited
        if self.md_for != subject || !self.md_editing {
            self.md_for = subject;
            self.md_buf = subject.map(|e| self.repo.str(e, "documentation")).unwrap_or_default();
        }
        let mut close = false;
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Markdown Documentation").size(13.0));
            ui.add_space(12.0);
            ui.radio_value(&mut self.md_preview, false, "Edit");
            ui.radio_value(&mut self.md_preview, true, "Preview");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("×").clicked() {
                    close = true;
                }
            });
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
                if self.md_preview {
                    self.md_editing = false;
                    let text = self.md_buf.clone();
                    preview(ui, &text);
                } else {
                    let r = ui.add_enabled(
                        subject.is_some(),
                        egui::TextEdit::multiline(&mut self.md_buf).code_editor().desired_width(f32::INFINITY).desired_rows(8),
                    );
                    self.md_editing = r.has_focus();
                    if let (true, Some(e)) = (r.lost_focus(), subject) {
                        if self.repo.str(e, "documentation") != self.md_buf {
                            let text = self.md_buf.clone();
                            self.change(|a| a.repo.set(e, "documentation", json!(text)));
                            // the Documentation editor shows the same text
                            self.doc_buf = self.md_buf.clone();
                        }
                    }
                }
            });
        });
        if close {
            self.show_markdown = false;
        }
    }
}
