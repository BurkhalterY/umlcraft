//! View > Markdown Documentation (Ctrl+Alt+D): a panel under the diagram to edit the selected
//! element's documentation as Markdown, or to preview it (extensions/default/markdown).
//!
//! The preview reads GitHub Flavored Markdown as the original's renderer (marked) does, with
//! pulldown-cmark: headings, paragraphs, nested bullet, numbered and task lists, block quotes, code
//! blocks, rules, tables, and **bold**, *italic*, ~~strikethrough~~, `code` and links, bare web
//! addresses included. Raw HTML is shown as text.

use crate::app::App;
use crate::model::Eid;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Stroke};
use serde_json::json;

/// A run of inline text with its styles.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    pub text: String,
    pub strong: bool,
    pub italic: bool,
    pub strike: bool,
    pub code: bool,
    /// The address of the link it is part of.
    pub link: Option<String>,
}

/// The mark in front of a list item.
#[derive(Clone, Debug, PartialEq)]
pub enum Mark {
    Bullet,
    Number(u64),
    Task(bool),
}

/// One block of the preview. `depth` is the number of lists it is in, `quote` of block quotes.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Text { heading: Option<u8>, runs: Vec<Run>, depth: usize, quote: usize, mark: Option<Mark> },
    Code { text: String, depth: usize, quote: usize },
    Rule { quote: usize },
    Table { head: Vec<Vec<Run>>, rows: Vec<Vec<Vec<Run>>>, depth: usize, quote: usize },
}

/// Split a Markdown text into the preview's blocks.
pub fn parse(text: &str) -> Vec<Block> {
    use pulldown_cmark::{Event, Tag, TagEnd};
    #[derive(Default)]
    struct State {
        out: Vec<Block>,
        runs: Vec<Run>,
        heading: Option<u8>,
        mark: Option<Mark>,
        lists: Vec<Option<u64>>,
        quote: usize,
        strong: usize,
        italic: usize,
        strike: usize,
        link: Vec<String>,
        code: Option<String>,
        table: Option<(Vec<Vec<Run>>, Vec<Vec<Vec<Run>>>, bool)>,
    }
    impl State {
        fn push(&mut self, text: &str, code: bool) {
            let run = Run { text: text.to_string(), strong: self.strong > 0, italic: self.italic > 0, strike: self.strike > 0, code, link: self.link.last().cloned() };
            // a run with the styles of the previous one extends it
            match self.runs.last_mut() {
                Some(last) if Run { text: String::new(), ..last.clone() } == Run { text: String::new(), ..run.clone() } => last.text.push_str(text),
                _ => self.runs.push(run),
            }
        }
        fn flush(&mut self) {
            if self.runs.is_empty() && self.mark.is_none() {
                return;
            }
            let runs = std::mem::take(&mut self.runs);
            self.out.push(Block::Text { heading: self.heading, runs, depth: self.lists.len(), quote: self.quote, mark: self.mark.take() });
        }
    }
    let mut s = State::default();
    for ev in events(text) {
        if let Some(c) = s.code.as_mut() {
            match ev {
                Event::Text(t) => c.push_str(&t),
                Event::End(TagEnd::CodeBlock) => {
                    let text = s.code.take().unwrap_or_default();
                    s.out.push(Block::Code { text: text.trim_end_matches('\n').to_string(), depth: s.lists.len(), quote: s.quote });
                }
                _ => {}
            }
            continue;
        }
        match ev {
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => s.flush(),
            Event::Start(Tag::Heading { level, .. }) => {
                s.flush();
                s.heading = Some(level as u8);
            }
            Event::End(TagEnd::Heading(_)) => {
                s.flush();
                s.heading = None;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                s.flush();
                s.quote += 1;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                s.flush();
                s.quote -= 1;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                s.flush();
                s.code = Some(String::new());
            }
            Event::Start(Tag::List(start)) => {
                s.flush();
                s.lists.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                s.flush();
                s.lists.pop();
            }
            Event::Start(Tag::Item) => {
                s.flush();
                s.mark = Some(match s.lists.last_mut() {
                    Some(Some(n)) => {
                        *n += 1;
                        Mark::Number(*n - 1)
                    }
                    _ => Mark::Bullet,
                });
            }
            Event::End(TagEnd::Item) => s.flush(),
            Event::TaskListMarker(done) => s.mark = Some(Mark::Task(done)),
            Event::Start(Tag::Table(_)) => {
                s.flush();
                s.table = Some((vec![], vec![], false));
            }
            Event::Start(Tag::TableHead) => {
                if let Some(t) = s.table.as_mut() {
                    t.2 = true;
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(t) = s.table.as_mut() {
                    t.2 = false;
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(t) = s.table.as_mut() {
                    t.1.push(vec![]);
                }
            }
            Event::Start(Tag::TableCell) => s.runs.clear(),
            Event::End(TagEnd::TableCell) => {
                let runs = std::mem::take(&mut s.runs);
                if let Some((head, rows, in_head)) = s.table.as_mut() {
                    if *in_head {
                        head.push(runs);
                    } else if let Some(row) = rows.last_mut() {
                        row.push(runs);
                    }
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some((head, rows, _)) = s.table.take() {
                    s.out.push(Block::Table { head, rows, depth: s.lists.len(), quote: s.quote });
                }
            }
            Event::Start(Tag::Strong) => s.strong += 1,
            Event::End(TagEnd::Strong) => s.strong -= 1,
            Event::Start(Tag::Emphasis) => s.italic += 1,
            Event::End(TagEnd::Emphasis) => s.italic -= 1,
            Event::Start(Tag::Strikethrough) => s.strike += 1,
            Event::End(TagEnd::Strikethrough) => s.strike -= 1,
            Event::Start(Tag::Link { dest_url, .. }) | Event::Start(Tag::Image { dest_url, .. }) => s.link.push(dest_url.to_string()),
            Event::End(TagEnd::Link | TagEnd::Image) => {
                s.link.pop();
            }
            Event::Text(t) => s.push(&t, false),
            Event::Code(t) => s.push(&t, true),
            Event::Html(t) | Event::InlineHtml(t) => s.push(&t, true),
            Event::End(TagEnd::HtmlBlock) => s.flush(),
            Event::SoftBreak => s.push(" ", false),
            Event::HardBreak => s.push("\n", false),
            Event::Rule => {
                s.flush();
                s.out.push(Block::Rule { quote: s.quote });
            }
            _ => {}
        }
    }
    s.flush();
    s.out
}

/// The runs of a block as a text layout in the theme's colours.
fn runs_job(ui: &egui::Ui, runs: &[Run], size: f32, strong_all: bool, quote: bool) -> LayoutJob {
    let v = ui.visuals();
    let base = if quote { v.weak_text_color() } else { v.text_color() };
    let mut job = LayoutJob::default();
    for r in runs {
        // no bold face in the UI font: the strong text colour stands for it
        let color = if r.link.is_some() { v.hyperlink_color } else if r.strong || strong_all { v.strong_text_color() } else { base };
        let mut f = TextFormat { font_id: FontId::proportional(size), color, italics: r.italic, ..Default::default() };
        if r.code {
            f.font_id = FontId::monospace(size - 1.0);
            f.background = v.code_bg_color;
        }
        if r.strike {
            f.strikethrough = Stroke::new(1.0, color);
        }
        if r.link.is_some() {
            f.underline = Stroke::new(1.0, color);
        }
        job.append(&r.text, 0.0, f);
    }
    job
}

/// Draw the preview of a Markdown text.
pub fn preview(ui: &mut egui::Ui, text: &str) {
    let bar = Color32::from_gray(120);
    for (i, b) in parse(text).into_iter().enumerate() {
        let (depth, quote) = match &b {
            Block::Text { depth, quote, .. } | Block::Code { depth, quote, .. } | Block::Table { depth, quote, .. } => (*depth, *quote),
            Block::Rule { quote } => (0, *quote),
        };
        let left = quote as f32 * 12.0 + depth as f32 * 18.0;
        let r = ui.horizontal_top(|ui| {
            ui.add_space(left);
            ui.vertical(|ui| match b {
                Block::Text { heading, runs, mark, .. } => {
                    let size = heading.map(|h| [22.0, 19.0, 16.0, 14.0, 13.0, 12.0][(h as usize).clamp(1, 6) - 1]).unwrap_or(13.0);
                    if heading.is_some() {
                        ui.add_space(4.0);
                    }
                    let job = runs_job(ui, &runs, size, heading.is_some(), quote > 0);
                    ui.horizontal_top(|ui| {
                        match mark {
                            Some(Mark::Bullet) => {
                                ui.label("•");
                            }
                            Some(Mark::Number(n)) => {
                                ui.label(format!("{n}."));
                            }
                            Some(Mark::Task(done)) => {
                                let mut d = done;
                                ui.add_enabled(false, egui::Checkbox::without_text(&mut d));
                            }
                            None => {}
                        }
                        ui.add(egui::Label::new(job).wrap());
                    });
                    if heading.map(|h| h <= 2).unwrap_or(false) {
                        let r = ui.max_rect();
                        ui.painter().hline(r.x_range(), ui.cursor().top(), Stroke::new(1.0, Color32::from_gray(90)));
                    }
                }
                Block::Code { text, .. } => {
                    egui::Frame::NONE.fill(ui.visuals().code_bg_color).inner_margin(6).show(ui, |ui| {
                        ui.label(egui::RichText::new(text).monospace());
                    });
                }
                Block::Rule { .. } => {
                    ui.separator();
                }
                Block::Table { head, rows, .. } => {
                    egui::Frame::NONE.stroke(Stroke::new(1.0, Color32::from_gray(90))).inner_margin(4).show(ui, |ui| {
                        egui::Grid::new(("markdown-table", i)).striped(true).spacing([14.0, 4.0]).show(ui, |ui| {
                            for cell in &head {
                                ui.label(runs_job(ui, cell, 13.0, true, quote > 0));
                            }
                            ui.end_row();
                            for row in &rows {
                                for cell in row {
                                    ui.label(runs_job(ui, cell, 13.0, false, quote > 0));
                                }
                                ui.end_row();
                            }
                        });
                    });
                }
            });
        });
        // a bar left of each quote level
        for q in 0..quote {
            let x = r.response.rect.left() + q as f32 * 12.0 + 3.0;
            ui.painter().vline(x, r.response.rect.y_range(), Stroke::new(2.0, bar));
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
