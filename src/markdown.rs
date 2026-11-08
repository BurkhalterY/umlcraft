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
                    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap: f32| {
                        let mut job = highlight(text.as_str());
                        job.wrap.max_width = wrap;
                        ui.fonts_mut(|f| f.layout_job(job))
                    };
                    let r = ui.add_enabled(
                        subject.is_some(),
                        egui::TextEdit::multiline(&mut self.md_buf)
                            .code_editor()
                            .layouter(&mut layouter)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, (ui.available_height() - 8.0).max(40.0))),
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

/// Markdown source with colours, for the editor: headings, quotes, list marks, fenced code, `code`,
/// **bold** and *italic* (the original uses CodeMirror's markdown mode; the colours are UMLCraft's).
pub fn highlight(text: &str) -> LayoutJob {
    let mono = FontId::monospace(12.0);
    let fmt = |c: Color32| TextFormat { font_id: mono.clone(), color: c, ..Default::default() };
    let (plain, heading, mark, code, strong) = (crate::ui::TEXT(), Color32::from_rgb(0x4f, 0x99, 0xff), Color32::from_rgb(0xd8, 0xa0, 0x3c), Color32::from_rgb(0x8f, 0xbf, 0x6f), Color32::from_rgb(0xe0, 0x7a, 0x9a));
    let mut job = LayoutJob::default();
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("```") {
            fenced = !fenced;
            job.append(line, 0.0, fmt(code));
            continue;
        }
        if fenced {
            job.append(line, 0.0, fmt(code));
            continue;
        }
        if t.starts_with('#') {
            job.append(line, 0.0, fmt(heading));
            continue;
        }
        if t.starts_with('>') {
            job.append(line, 0.0, fmt(mark));
            continue;
        }
        // a list mark, then the inline spans
        let indent = line.len() - t.len();
        let mut rest = line;
        let list = ["- ", "* ", "+ "].iter().any(|m| t.starts_with(m)) || t.split_once(". ").map(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())).unwrap_or(false);
        if list {
            let end = indent + t.find(' ').unwrap_or(0) + 1;
            job.append(&line[..end], 0.0, fmt(mark));
            rest = &line[end..];
        }
        // inline: `code`, **bold**, *italic*
        let mut i = 0;
        let b = rest.as_bytes();
        let mut start = 0;
        while i < b.len() {
            let (open, color) = if b[i] == b'`' { ("`", code) } else if rest[i..].starts_with("**") { ("**", strong) } else if b[i] == b'*' || b[i] == b'_' { (&rest[i..i + 1], strong) } else { ("", plain) };
            if !open.is_empty() {
                if let Some(k) = rest[i + open.len()..].find(open).filter(|k| *k > 0) {
                    let end = i + open.len() + k + open.len();
                    job.append(&rest[start..i], 0.0, fmt(plain));
                    job.append(&rest[i..end], 0.0, fmt(color));
                    i = end;
                    start = end;
                    continue;
                }
            }
            i += rest[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        }
        job.append(&rest[start..], 0.0, fmt(plain));
    }
    job
}

/// GitHub Flavored Markdown as the original's renderer (marked, with its GFM defaults) reads it:
/// tables, ~~strikethrough~~, task lists and GFM block quotes.
pub fn gfm_options() -> pulldown_cmark::Options {
    use pulldown_cmark::Options;
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS | Options::ENABLE_GFM
}

/// The parsed events of a Markdown text, with bare `http://`, `https://` and `www.` addresses
/// outside links and code turned into links (GFM autolinks).
pub fn events(text: &str) -> Vec<pulldown_cmark::Event<'_>> {
    use pulldown_cmark::{Event, Tag, TagEnd};
    let mut out = vec![];
    let (mut in_link, mut in_code) = (0, 0);
    for ev in pulldown_cmark::Parser::new_ext(text, gfm_options()) {
        match &ev {
            Event::Start(Tag::Link { .. } | Tag::Image { .. }) => in_link += 1,
            Event::End(TagEnd::Link | TagEnd::Image) => in_link -= 1,
            Event::Start(Tag::CodeBlock(_)) => in_code += 1,
            Event::End(TagEnd::CodeBlock) => in_code -= 1,
            _ => {}
        }
        match ev {
            Event::Text(t) if in_link == 0 && in_code == 0 => autolink(&t, &mut out),
            ev => out.push(ev),
        }
    }
    out
}

/// Split a text around the web addresses in it (the GFM "extended autolink" rule, simplified: an
/// address starts after a space, an opening bracket or the start, and runs to the next space or `<`,
/// without trailing punctuation or an unbalanced `)`).
fn autolink(t: &str, out: &mut Vec<pulldown_cmark::Event<'_>>) {
    use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};
    let mut rest = t;
    loop {
        let start = ["https://", "http://", "www."]
            .iter()
            .filter_map(|p| {
                let mut from = 0;
                while let Some(i) = rest[from..].find(p).map(|i| i + from) {
                    let before = rest[..i].chars().next_back();
                    if before.map(|c| c.is_whitespace() || "(*_~".contains(c)).unwrap_or(true) {
                        return Some(i);
                    }
                    from = i + p.len();
                }
                None
            })
            .min();
        let Some(i) = start else { break };
        let mut end = rest[i..].find(|c: char| c.is_whitespace() || c == '<').map(|k| i + k).unwrap_or(rest.len());
        loop {
            let url = &rest[i..end];
            let Some(last) = url.chars().next_back() else { break };
            let unbalanced = last == ')' && url.matches(')').count() > url.matches('(').count();
            if "?!.,:*_~'\"".contains(last) || unbalanced {
                end -= last.len_utf8();
            } else {
                break;
            }
        }
        let url = &rest[i..end];
        let bare = url.trim_start_matches("https://").trim_start_matches("http://").trim_start_matches("www.");
        if bare.is_empty() {
            // a prefix alone stays text
            let stop = i + url.len().max(1);
            out.push(Event::Text(CowStr::from(rest[..stop].to_string())));
            rest = &rest[stop..];
            continue;
        }
        if i > 0 {
            out.push(Event::Text(CowStr::from(rest[..i].to_string())));
        }
        let dest = if url.starts_with("www.") { format!("http://{url}") } else { url.to_string() };
        out.push(Event::Start(Tag::Link { link_type: LinkType::Autolink, dest_url: CowStr::from(dest), title: CowStr::from(""), id: CowStr::from("") }));
        out.push(Event::Text(CowStr::from(url.to_string())));
        out.push(Event::End(TagEnd::Link));
        rest = &rest[end..];
    }
    if !rest.is_empty() {
        out.push(Event::Text(CowStr::from(rest.to_string())));
    }
}
