//! The user's keymap: `keymap.json` in UMLCraft's settings folder maps keystrokes to command ids, for
//! example `{"cmdctrl-alt-g": "view:show-grid"}`. Its bindings win over the built-in shortcuts
//! (keymap-manager.js:49-54, 154-292; app-context.js:760-764). A file that cannot be read is
//! reported on the console and ignored.

use crate::app::App;
use egui::{Key, Modifiers};
use std::path::Path;

/// A parsed keystroke: modifiers and key.
pub type Stroke = (Modifiers, Key);

/// Parse a descriptor such as `cmdctrl-alt-g`, `ctrl-shift-f4` or `alt--` (the minus key). Words are
/// case-insensitive; `cmdctrl` is Cmd on macOS and Ctrl elsewhere.
pub fn parse_stroke(desc: &str) -> Option<Stroke> {
    let d = desc.trim().to_lowercase();
    // the last part is the key; a trailing "-" after a separator is the minus key
    let (mods, key) = if let Some(m) = d.strip_suffix("--") {
        (m, "-")
    } else if d == "-" {
        ("", "-")
    } else {
        match d.rsplit_once('-') {
            Some((m, k)) => (m, k),
            None => ("", d.as_str()),
        }
    };
    let mut m = Modifiers::NONE;
    for part in mods.split('-').filter(|p| !p.is_empty()) {
        match part {
            "cmdctrl" | "mod" => m = m | Modifiers::COMMAND,
            "cmd" if cfg!(target_os = "macos") => m = m | Modifiers::MAC_CMD | Modifiers::COMMAND,
            "cmd" => return None,
            "ctrl" if cfg!(target_os = "macos") => m = m | Modifiers::CTRL,
            "ctrl" => m = m | Modifiers::COMMAND,
            "alt" | "option" => m = m | Modifiers::ALT,
            "shift" => m = m | Modifiers::SHIFT,
            _ => return None,
        }
    }
    let k = match key {
        "up" => Key::ArrowUp,
        "down" => Key::ArrowDown,
        "left" => Key::ArrowLeft,
        "right" => Key::ArrowRight,
        "enter" | "return" => Key::Enter,
        "escape" | "esc" => Key::Escape,
        // macOS calls the Backspace key Delete (keymap-manager.js:183-185)
        "delete" if cfg!(target_os = "macos") => Key::Backspace,
        "delete" => Key::Delete,
        "backspace" => Key::Backspace,
        "tab" => Key::Tab,
        "space" => Key::Space,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "=" | "plus" => Key::Equals,
        "-" | "minus" => Key::Minus,
        "," => Key::Comma,
        "." => Key::Period,
        "/" => Key::Slash,
        ";" => Key::Semicolon,
        "[" => Key::OpenBracket,
        "]" => Key::CloseBracket,
        "`" => Key::Backtick,
        "'" => Key::Quote,
        "\\" => Key::Backslash,
        k => Key::from_name(&k.to_uppercase()).or_else(|| Key::from_name(k))?,
    };
    Some((m, k))
}

/// Built-in bindings of commands the Command Palette does not list (keymaps/linux.json,
/// uml/keymaps/uml.json), in the palette's "Ctrl+Shift+X" form.
const EXTRA: [(&str, &str); 9] = [
    ("find:find", "Ctrl+F"),
    ("application:quit", "Ctrl+Q"),
    ("view:command-palette", "Ctrl+Shift+P"),
    ("format:stereotype-none", "Ctrl+Shift+0"),
    ("format:stereotype-label", "Ctrl+Shift+1"),
    ("format:stereotype-decoration", "Ctrl+Shift+2"),
    ("format:stereotype-decoration-label", "Ctrl+Shift+3"),
    ("format:stereotype-icon", "Ctrl+Shift+4"),
    ("format:stereotype-icon-label", "Ctrl+Shift+5"),
];

/// A key as the palette writes it ("Ctrl+Shift+S", "F4", "Ctrl+-"), where Ctrl is the command key.
fn parse_label(label: &str) -> Option<Stroke> {
    let d = label.replace("Ctrl+", "cmdctrl+").replace('+', "-").replace("--", "-minus").replace("-=", "-plus");
    parse_stroke(if d == "-" { "minus" } else { &d })
}

/// The built-in keymap: the platform keys, then the UML ones, as (keystroke, command id).
pub fn defaults() -> Vec<(Stroke, &'static str)> {
    let palette = crate::palette::COMMANDS.iter().map(|c| (c.key, c.id));
    let mut out: Vec<(Stroke, &'static str)> = palette.chain(EXTRA.iter().map(|(id, k)| (*k, *id))).filter(|(k, _)| !k.is_empty()).filter_map(|(k, id)| Some((parse_label(k)?, id))).collect();
    // macOS only: "cmd-," opens Preferences (keymaps/darwin.json:6)
    if cfg!(target_os = "macos") {
        out.extend(parse_stroke("cmd-,").map(|s| (s, "application:preferences")));
    }
    out
}

/// A keystroke as menus show it: "Ctrl+Alt+Shift+K" in descriptor order, or on macOS the
/// symbols run together ("⌘⌥⇧K") (keymap-manager.js:75-152; menu-helpers.js:128-150).
pub fn stroke_label((m, k): Stroke, mac: bool) -> String {
    let key = match k {
        Key::ArrowUp => "↑".to_string(),
        Key::ArrowDown => "↓".to_string(),
        Key::ArrowLeft => "←".to_string(),
        Key::ArrowRight => "→".to_string(),
        Key::Backspace if mac => "⌫".to_string(),
        Key::Equals | Key::Plus => "=".to_string(),
        Key::Minus => "-".to_string(),
        k => k.symbol_or_name().to_string(),
    };
    let mut parts: Vec<&str> = vec![];
    if mac {
        if m.mac_cmd || m.command {
            parts.push("⌘");
        }
        if m.ctrl && !m.command {
            parts.push("⌃");
        }
        if m.alt {
            parts.push("⌥");
        }
        if m.shift {
            parts.push("⇧");
        }
        return parts.concat() + &key;
    }
    if m.command || m.ctrl {
        parts.push("Ctrl");
    }
    if m.alt {
        parts.push("Alt");
    }
    if m.shift {
        parts.push("Shift");
    }
    parts.push(&key);
    parts.join("+")
}

/// The keys a text field keeps for itself: Ctrl+C, X, V, Z, Y and A, Delete and Backspace,
/// whatever else is held (keymap-manager.js:305-316). Ctrl is the command key.
pub fn is_edit_key(m: Modifiers, k: Key) -> bool {
    (m.command && matches!(k, Key::C | Key::X | Key::V | Key::Z | Key::Y | Key::A)) || matches!(k, Key::Delete | Key::Backspace)
}

/// Read a keymap file: keystroke descriptors to command ids. Bad entries are skipped.
pub fn load(path: &Path) -> Result<Vec<(Stroke, String)>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let obj = v.as_object().ok_or("the keymap is not a JSON object")?;
    Ok(obj.iter().filter_map(|(k, cmd)| Some((parse_stroke(k)?, cmd.as_str()?.to_string()))).collect())
}

impl App {
    /// The final keymap: the built-in bindings, then the user's, each replacing what an earlier
    /// source bound to the same keystroke (keymap-manager.js:49-54; app-context.js:760-764).
    pub fn keymap(&self) -> Vec<(Stroke, String)> {
        let mut out: Vec<(Stroke, String)> = defaults().into_iter().map(|(s, id)| (s, id.to_string())).collect();
        for (s, id) in &self.user_keymap {
            out.retain(|(o, _)| !(o.0.matches_exact(s.0) && o.1 == s.1));
            out.push((*s, id.clone()));
        }
        out
    }

    /// The keystroke a menu shows for a command: the last one bound to it in the final keymap, or
    /// nothing (menu-manager.js:55-62; menu-helpers.js:111-150).
    pub fn accel(&self, id: &str) -> String {
        self.keymap().into_iter().rev().find(|(_, c)| c == id).map(|(s, _)| stroke_label(s, cfg!(target_os = "macos"))).unwrap_or_default()
    }

    /// A menu button with the command's keystroke on its right.
    pub fn key_button(&self, name: &str, id: &str) -> egui::Button<'static> {
        egui::Button::new(name.to_string()).shortcut_text(self.accel(id))
    }

    /// A check item's text: the name, then the command's keystroke.
    pub fn key_text(&self, name: &str, id: &str) -> String {
        let a = self.accel(id);
        if a.is_empty() {
            name.to_string()
        } else {
            format!("{name}    {a}")
        }
    }

    /// Load `keymap.json` next to the preferences, when there is one.
    pub fn load_user_keymap(&mut self, dir: &Path) {
        let path = dir.join("keymap.json");
        if !path.exists() {
            return;
        }
        match load(&path) {
            Ok(k) => self.user_keymap = k,
            Err(e) => eprintln!("[UMLCraft] {}: {e}", path.display()),
        }
    }

    /// Run the user's bindings first, consuming the keys they use (exact modifiers only). While a
    /// text field has the keys, its edit keys are left to it.
    pub fn user_shortcuts(&mut self, ctx: &egui::Context, typing: bool) {
        if self.user_keymap.is_empty() {
            return;
        }
        let bindings = self.user_keymap.clone();
        for ((mods, key), cmd) in bindings {
            if typing && is_edit_key(mods, key) {
                continue;
            }
            let hit = ctx.input_mut(|i| {
                let n = i.events.len();
                i.events.retain(|e| !matches!(e, egui::Event::Key { key: k, pressed: true, modifiers, .. } if *k == key && modifiers.matches_exact(mods)));
                n != i.events.len()
            });
            if hit {
                self.run_command(ctx, &cmd);
            }
        }
    }
}
