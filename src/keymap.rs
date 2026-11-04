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

/// Read a keymap file: keystroke descriptors to command ids. Bad entries are skipped.
pub fn load(path: &Path) -> Result<Vec<(Stroke, String)>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let obj = v.as_object().ok_or("the keymap is not a JSON object")?;
    Ok(obj.iter().filter_map(|(k, cmd)| Some((parse_stroke(k)?, cmd.as_str()?.to_string()))).collect())
}

impl App {
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

    /// Run the user's bindings first, consuming the keys they use (exact modifiers only).
    pub fn user_shortcuts(&mut self, ctx: &egui::Context) {
        if self.user_keymap.is_empty() {
            return;
        }
        let bindings = self.user_keymap.clone();
        for ((mods, key), cmd) in bindings {
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
