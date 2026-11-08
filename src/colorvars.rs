//! StarUML 8's colour variables: `$background`, `$foreground`, `$transparent` and `$<hue><1-12>`,
//! drawn with the colours of the current theme (https://docs.staruml.io/user-guide/formatting-diagram/).
//!
//! StarUML does not publish the values behind the variables. The docs give the names (13 hues of 12
//! steps) and say `$background` and `$foreground` are white and black in the light theme. UMLCraft
//! fills in the rest with the Radix Colors scales (MIT licence, © 2021 Radix, @radix-ui/colors 3.0.0),
//! whose 12-step scales carry the same 13 names: the light theme takes each scale's light values and the
//! dark theme its dark ones. The dark theme's `$background` and `$foreground` are gray 2 and gray 12 of
//! the dark gray scale. See MECHANICS.md "Colour variables".
//!
//! Files stay readable by StarUML 7 unless the person converts a project: plain projects keep `#rrggbb`
//! colours, File › Export › StarUML V7 writes a copy with every variable replaced by its light colour.

use crate::model::{Eid, Repo};
use crate::render::Rgba;
use serde_json::json;
use std::cell::Cell;

/// The 13 hues, in the order of the docs and of the Color dialog's rows.
pub const HUES: [&str; 13] = ["slate", "gray", "red", "pink", "purple", "blue", "cyan", "green", "brown", "orange", "yellow", "lime", "mint"];

/// The colour fields of a view that may hold a variable.
pub const COLOR_FIELDS: [&str; 3] = ["fillColor", "lineColor", "fontColor"];

/// Steps 1 to 12 of each hue in the light theme (Radix Colors, light scales).
pub const LIGHT: [[u32; 12]; 13] = [
    [0xfcfcfd, 0xf9f9fb, 0xf0f0f3, 0xe8e8ec, 0xe0e1e6, 0xd9d9e0, 0xcdced6, 0xb9bbc6, 0x8b8d98, 0x80838d, 0x60646c, 0x1c2024], // slate
    [0xfcfcfc, 0xf9f9f9, 0xf0f0f0, 0xe8e8e8, 0xe0e0e0, 0xd9d9d9, 0xcecece, 0xbbbbbb, 0x8d8d8d, 0x838383, 0x646464, 0x202020], // gray
    [0xfffcfc, 0xfff7f7, 0xfeebec, 0xffdbdc, 0xffcdce, 0xfdbdbe, 0xf4a9aa, 0xeb8e90, 0xe5484d, 0xdc3e42, 0xce2c31, 0x641723], // red
    [0xfffcfe, 0xfef7fb, 0xfee9f5, 0xfbdcef, 0xf6cee7, 0xefbfdd, 0xe7acd0, 0xdd93c2, 0xd6409f, 0xcf3897, 0xc2298a, 0x651249], // pink
    [0xfefcfe, 0xfbf7fe, 0xf7edfe, 0xf2e2fc, 0xead5f9, 0xe0c4f4, 0xd1afec, 0xbe93e4, 0x8e4ec6, 0x8347b9, 0x8145b5, 0x402060], // purple
    [0xfbfdff, 0xf4faff, 0xe6f4fe, 0xd5efff, 0xc2e5ff, 0xacd8fc, 0x8ec8f6, 0x5eb1ef, 0x0090ff, 0x0588f0, 0x0d74ce, 0x113264], // blue
    [0xfafdfe, 0xf2fafb, 0xdef7f9, 0xcaf1f6, 0xb5e9f0, 0x9ddde7, 0x7dcedc, 0x3db9cf, 0x00a2c7, 0x0797b9, 0x107d98, 0x0d3c48], // cyan
    [0xfbfefc, 0xf4fbf6, 0xe6f6eb, 0xd6f1df, 0xc4e8d1, 0xadddc0, 0x8eceaa, 0x5bb98b, 0x30a46c, 0x2b9a66, 0x218358, 0x193b2d], // green
    [0xfefdfc, 0xfcf9f6, 0xf6eee7, 0xf0e4d9, 0xebdaca, 0xe4cdb7, 0xdcbc9f, 0xcea37e, 0xad7f58, 0xa07553, 0x815e46, 0x3e332e], // brown
    [0xfefcfb, 0xfff7ed, 0xffefd6, 0xffdfb5, 0xffd19a, 0xffc182, 0xf5ae73, 0xec9455, 0xf76b15, 0xef5f00, 0xcc4e00, 0x582d1d], // orange
    [0xfdfdf9, 0xfefce9, 0xfffab8, 0xfff394, 0xffe770, 0xf3d768, 0xe4c767, 0xd5ae39, 0xffe629, 0xffdc00, 0x9e6c00, 0x473b1f], // yellow
    [0xfcfdfa, 0xf8faf3, 0xeef6d6, 0xe2f0bd, 0xd3e7a6, 0xc2da91, 0xabc978, 0x8db654, 0xbdee63, 0xb0e64c, 0x5c7c2f, 0x37401c], // lime
    [0xf9fefd, 0xf2fbf9, 0xddf9f2, 0xc8f4e9, 0xb3ecde, 0x9ce0d0, 0x7ecfbd, 0x4cbba5, 0x86ead4, 0x7de0cb, 0x027864, 0x16433c], // mint
];

/// Steps 1 to 12 of each hue in the dark theme (Radix Colors, dark scales).
pub const DARK: [[u32; 12]; 13] = [
    [0x111113, 0x18191b, 0x212225, 0x272a2d, 0x2e3135, 0x363a3f, 0x43484e, 0x5a6169, 0x696e77, 0x777b84, 0xb0b4ba, 0xedeef0], // slate
    [0x111111, 0x191919, 0x222222, 0x2a2a2a, 0x313131, 0x3a3a3a, 0x484848, 0x606060, 0x6e6e6e, 0x7b7b7b, 0xb4b4b4, 0xeeeeee], // gray
    [0x191111, 0x201314, 0x3b1219, 0x500f1c, 0x611623, 0x72232d, 0x8c333a, 0xb54548, 0xe5484d, 0xec5d5e, 0xff9592, 0xffd1d9], // red
    [0x191117, 0x21121d, 0x37172f, 0x4b143d, 0x591c47, 0x692955, 0x833869, 0xa84885, 0xd6409f, 0xde51a8, 0xff8dcc, 0xfdd1ea], // pink
    [0x18111b, 0x1e1523, 0x301c3b, 0x3d224e, 0x48295c, 0x54346b, 0x664282, 0x8457aa, 0x8e4ec6, 0x9a5cd0, 0xd19dff, 0xecd9fa], // purple
    [0x0d1520, 0x111927, 0x0d2847, 0x003362, 0x004074, 0x104d87, 0x205d9e, 0x2870bd, 0x0090ff, 0x3b9eff, 0x70b8ff, 0xc2e6ff], // blue
    [0x0b161a, 0x101b20, 0x082c36, 0x003848, 0x004558, 0x045468, 0x12677e, 0x11809c, 0x00a2c7, 0x23afd0, 0x4ccce6, 0xb6ecf7], // cyan
    [0x0e1512, 0x121b17, 0x132d21, 0x113b29, 0x174933, 0x20573e, 0x28684a, 0x2f7c57, 0x30a46c, 0x33b074, 0x3dd68c, 0xb1f1cb], // green
    [0x12110f, 0x1c1816, 0x28211d, 0x322922, 0x3e3128, 0x4d3c2f, 0x614a39, 0x7c5f46, 0xad7f58, 0xb88c67, 0xdbb594, 0xf2e1ca], // brown
    [0x17120e, 0x1e160f, 0x331e0b, 0x462100, 0x562800, 0x66350c, 0x7e451d, 0xa35829, 0xf76b15, 0xff801f, 0xffa057, 0xffe0c2], // orange
    [0x14120b, 0x1b180f, 0x2d2305, 0x362b00, 0x433500, 0x524202, 0x665417, 0x836a21, 0xffe629, 0xffff57, 0xf5e147, 0xf6eeb4], // yellow
    [0x11130c, 0x151a10, 0x1f2917, 0x29371d, 0x334423, 0x3d522a, 0x496231, 0x577538, 0xbdee63, 0xd4ff70, 0xbde56c, 0xe3f7ba], // lime
    [0x0e1515, 0x0f1b1b, 0x092c2b, 0x003a38, 0x004744, 0x105650, 0x1e685f, 0x277f70, 0x86ead4, 0xa8f5e5, 0x58d5ba, 0xc4f5e1], // mint
];

/// `$background` and `$foreground`: white and black in the light theme (the docs), gray 2 and gray 12
/// of the dark scale in the dark theme (UMLCraft's choice).
pub const LIGHT_BACKGROUND: Rgba = Rgba(0xff, 0xff, 0xff, 255);
pub const LIGHT_FOREGROUND: Rgba = Rgba(0x00, 0x00, 0x00, 255);
pub const DARK_BACKGROUND: Rgba = Rgba(0x19, 0x19, 0x19, 255);
pub const DARK_FOREGROUND: Rgba = Rgba(0xee, 0xee, 0xee, 255);
/// `$transparent`, written to a StarUML 7 file as `#ffffff00` (the Color dialog's Transparent swatch).
pub const TRANSPARENT: Rgba = Rgba(0xff, 0xff, 0xff, 0);

thread_local! {
    static THEME: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Whether variables are drawn in their dark-theme colours: the interface theme, unless a caller has
/// fixed the theme for a while with `with_theme`.
pub fn dark() -> bool {
    THEME.with(|t| t.get()).unwrap_or_else(|| !crate::ui::light_theme())
}

/// Run `f` with variables drawn in the dark (`true`) or light theme, on this thread.
pub fn with_theme<T>(dark: bool, f: impl FnOnce() -> T) -> T {
    let before = THEME.with(|t| t.replace(Some(dark)));
    let out = f();
    THEME.with(|t| t.set(before));
    out
}

fn rgb(v: u32) -> Rgba {
    Rgba((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}

/// The colour of a variable (`$blue9`, any case) in the given theme; None for an unknown name.
pub fn resolve(name: &str, dark: bool) -> Option<Rgba> {
    let n = name.trim().strip_prefix('$')?.to_ascii_lowercase();
    match n.as_str() {
        "background" => return Some(if dark { DARK_BACKGROUND } else { LIGHT_BACKGROUND }),
        "foreground" => return Some(if dark { DARK_FOREGROUND } else { LIGHT_FOREGROUND }),
        "transparent" => return Some(TRANSPARENT),
        _ => {}
    }
    let (h, rest) = HUES.iter().enumerate().find_map(|(i, h)| n.strip_prefix(h).map(|r| (i, r)))?;
    let step: usize = rest.parse().ok().filter(|s| (1..=12).contains(s))?;
    Some(rgb(if dark { DARK } else { LIGHT }[h][step - 1]))
}

/// Whether a colour value is a variable.
pub fn is_var(s: &str) -> bool {
    s.trim_start().starts_with('$')
}

/// Every variable, in the Color dialog's order: background, foreground, transparent, then the 12
/// steps of each hue.
pub fn names() -> Vec<String> {
    let mut v = vec!["$background".to_string(), "$foreground".into(), "$transparent".into()];
    for h in HUES {
        for s in 1..=12 {
            v.push(format!("${h}{s}"));
        }
    }
    v
}

/// The variable most like a fixed colour, by its light-theme value ("each color is converted to the most
/// similar color variable", StarUML 8.0 release notes). A fully transparent colour becomes
/// `$transparent`; a partly transparent one has no variable and stays as it is.
pub fn nearest(c: Rgba) -> Option<String> {
    if c.3 == 0 {
        return Some("$transparent".into());
    }
    if c.3 != 255 {
        return None;
    }
    // the "redmean" weighting: close to how far apart people see two colours, and cheap
    let dist = |a: Rgba| {
        let rm = (c.0 as f64 + a.0 as f64) / 2.0;
        let (dr, dg, db) = (c.0 as f64 - a.0 as f64, c.1 as f64 - a.1 as f64, c.2 as f64 - a.2 as f64);
        (2.0 + rm / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rm) / 256.0) * db * db
    };
    names()
        .into_iter()
        .filter(|n| n != "$transparent")
        .map(|n| {
            let d = dist(resolve(&n, false).unwrap_or(LIGHT_BACKGROUND));
            (n, d)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(n, _)| n)
}

/// The light-theme colour of a variable as `#rrggbb` (`#ffffff00` for `$transparent`): what StarUML 7
/// gets in its place.
pub fn light_hex(name: &str) -> Option<String> {
    resolve(name, false).map(crate::style::hex)
}

fn color_values(r: &Repo, e: Eid) -> impl Iterator<Item = (&'static str, String)> + '_ {
    COLOR_FIELDS.into_iter().filter_map(move |f| r.raw(e, f).and_then(|v| v.as_str()).map(|s| (f, s.to_string())))
}

/// Whether any view of the project holds a colour variable.
pub fn uses_vars(r: &Repo) -> bool {
    r.all().any(|e| color_values(r, e).any(|(_, s)| is_var(&s)))
}

/// Whether the project has views with fixed colours, the kind a file of StarUML 7 or earlier holds.
pub fn has_fixed_colors(r: &Repo) -> bool {
    // a field left out of the file holds its default, a fixed colour too
    r.all_of("View").into_iter().any(|v| COLOR_FIELDS.iter().any(|f| crate::meta::order_of(r.ty(v), f).is_some() && !is_var(&r.str(v, f))))
}

/// Turn the fixed colours of these views into their most similar variables; a colour that has none
/// (partly transparent, or not a colour) is kept.
pub fn convert_views(r: &mut Repo, views: &[Eid]) {
    for &v in views {
        for f in COLOR_FIELDS {
            if crate::meta::order_of(r.ty(v), f).is_none() {
                continue;
            }
            let s = r.str(v, f);
            if is_var(&s) {
                continue;
            }
            let Some(c) = parse_fixed(&s) else { continue };
            if let Some(n) = nearest(c) {
                r.set(v, f, json!(n));
            }
        }
    }
}

/// The Convert answer: every view of the project.
pub fn convert(r: &mut Repo) {
    let views = r.all_of("View");
    convert_views(r, &views);
}

/// A copy of the project for StarUML 7: every variable replaced by its light-theme colour; an unknown
/// variable becomes the field's default.
pub fn to_v7(r: &Repo) -> Repo {
    let mut out = r.clone();
    for e in out.all().collect::<Vec<_>>() {
        for (f, s) in color_values(r, e).collect::<Vec<_>>() {
            if is_var(&s) {
                let hex = light_hex(&s).unwrap_or_else(|| if f == "fillColor" { "#ffffff".into() } else { "#000000".into() });
                out.set(e, f, json!(hex));
            }
        }
    }
    out
}

/// A fixed colour (not a variable), as `parse_color` reads it.
fn parse_fixed(s: &str) -> Option<Rgba> {
    const NONE: Rgba = Rgba(1, 2, 3, 4);
    let c = crate::render::parse_color(s, NONE);
    (c != NONE).then_some(c)
}

impl crate::app::App {
    /// Convert: every fixed colour of the project becomes its most similar variable, as one undo step;
    /// from then on new views get variables too.
    pub fn convert_colors(&mut self) {
        self.color_prompt = false;
        self.change(|a| convert(&mut a.repo));
        self.color_vars = true;
    }

    /// Keep Colors: the project stays as it was read.
    pub fn keep_colors(&mut self) {
        self.color_prompt = false;
    }

    /// A new view of a converted project takes variables in place of its fixed default colours, as
    /// StarUML 8 stores them; a plain project keeps `#rrggbb` so that StarUML 7 still opens it.
    pub fn apply_color_vars(&mut self, v: Eid) {
        if !self.color_vars || !self.repo.alive(v) {
            return;
        }
        let mut all = vec![];
        self.repo.descendants(v, &mut all);
        let views: Vec<Eid> = all.into_iter().filter(|e| crate::meta::is_kind_of(self.repo.ty(*e), "View")).collect();
        convert_views(&mut self.repo, &views);
    }

    /// File › Export › StarUML V7: the project as StarUML 7 reads it, with every variable replaced by its
    /// light-theme colour. The working project and its file are not changed.
    pub fn v7_json(&mut self, ms: &dyn crate::views::Measure) -> String {
        self.prepare_for_save(ms);
        to_v7(&self.repo).serialize()
    }

    /// The Convert / Keep Colors question (https://docs.staruml.io/user-guide/managing-project/).
    /// Enter converts, Esc keeps the colours.
    pub fn color_prompt_ui(&mut self, ctx: &egui::Context) {
        if !self.color_prompt {
            return;
        }
        if self.screenshot.is_some() {
            self.color_prompt = false;
            return;
        }
        let (mut convert, mut keep) = (false, false);
        egui::Modal::new(egui::Id::new("color-prompt")).show(ctx, |ui| {
            ui.set_max_width(440.0);
            ui.label(egui::RichText::new("Convert colors?").strong());
            ui.add_space(4.0);
            ui.label("This file was saved by an older version of StarUML. Convert the colors of views into color variables, so that they follow the light and dark themes?");
            ui.add_space(2.0);
            ui.label(egui::RichText::new("A converted project may not be drawn correctly by StarUML 7 or earlier; File › Export › StarUML V7... writes a copy it can open.").small());
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let b = ui.button("Convert");
                crate::ui::focus_primary(ui, &b);
                convert = b.clicked();
                keep = ui.button("Keep Colors").clicked();
            });
            convert |= ui.input(|i| i.key_pressed(egui::Key::Enter));
            keep |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if convert {
            self.convert_colors();
        } else if keep {
            self.keep_colors();
        }
    }
}
