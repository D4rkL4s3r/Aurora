//! Raccourcis clavier reconfigurables, persistés dans
//! `%APPDATA%\aurora\shortcuts.conf` (format `action=Ctrl+X`, une ligne par
//! action). Les actions sans fichier de config gardent leur valeur par défaut.

use egui::{Key, KeyboardShortcut, Modifiers};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ShortcutAction {
    SelectAll,
    Copy,
    Cut,
    Paste,
    Delete,
    Rename,
    AddressBar,
    ToggleSplit,
    NewTab,
    CloseTab,
    NextTab,
}

impl ShortcutAction {
    pub(crate) const ALL: [ShortcutAction; 11] = [
        Self::SelectAll,
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::Delete,
        Self::Rename,
        Self::AddressBar,
        Self::ToggleSplit,
        Self::NewTab,
        Self::CloseTab,
        Self::NextTab,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::SelectAll => "Tout sélectionner",
            Self::Copy => "Copier",
            Self::Cut => "Couper",
            Self::Paste => "Coller",
            Self::Delete => "Supprimer",
            Self::Rename => "Renommer",
            Self::AddressBar => "Barre d'adresse",
            Self::ToggleSplit => "Vue divisée",
            Self::NewTab => "Nouvel onglet",
            Self::CloseTab => "Fermer l'onglet",
            Self::NextTab => "Onglet suivant",
        }
    }

    /// Identifiant stable utilisé comme clé dans le fichier de config.
    fn config_key(self) -> &'static str {
        match self {
            Self::SelectAll => "select_all",
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste => "paste",
            Self::Delete => "delete",
            Self::Rename => "rename",
            Self::AddressBar => "address_bar",
            Self::ToggleSplit => "toggle_split",
            Self::NewTab => "new_tab",
            Self::CloseTab => "close_tab",
            Self::NextTab => "next_tab",
        }
    }

    fn default_binding(self) -> KeyboardShortcut {
        let ctrl = Modifiers::CTRL;
        let none = Modifiers::NONE;
        match self {
            Self::SelectAll => KeyboardShortcut::new(ctrl, Key::A),
            Self::Copy => KeyboardShortcut::new(ctrl, Key::C),
            Self::Cut => KeyboardShortcut::new(ctrl, Key::X),
            Self::Paste => KeyboardShortcut::new(ctrl, Key::V),
            Self::Delete => KeyboardShortcut::new(none, Key::Delete),
            Self::Rename => KeyboardShortcut::new(none, Key::F2),
            Self::AddressBar => KeyboardShortcut::new(ctrl, Key::L),
            Self::ToggleSplit => KeyboardShortcut::new(ctrl, Key::D),
            Self::NewTab => KeyboardShortcut::new(ctrl, Key::T),
            Self::CloseTab => KeyboardShortcut::new(ctrl, Key::W),
            Self::NextTab => KeyboardShortcut::new(ctrl, Key::Tab),
        }
    }
}

/// État de l'éditeur de raccourcis (fenêtre ⚙).
#[derive(Default)]
pub(crate) struct ShortcutsEditor {
    /// Action en attente d'une nouvelle combinaison de touches.
    pub(crate) listening: Option<ShortcutAction>,
}

pub(crate) struct ShortcutMap {
    /// `None` = action volontairement sans raccourci (délogée par un doublon).
    bindings: HashMap<ShortcutAction, Option<KeyboardShortcut>>,
}

impl ShortcutMap {
    fn defaults() -> HashMap<ShortcutAction, Option<KeyboardShortcut>> {
        ShortcutAction::ALL
            .iter()
            .map(|&a| (a, Some(a.default_binding())))
            .collect()
    }

    pub(crate) fn load() -> Self {
        let mut bindings = Self::defaults();
        if let Some(path) = config_path() {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let Some((key, value)) = line.split_once('=') else {
                        continue;
                    };
                    let Some(action) = ShortcutAction::ALL
                        .iter()
                        .find(|a| a.config_key() == key.trim())
                    else {
                        continue;
                    };
                    let value = value.trim();
                    if value.is_empty() {
                        bindings.insert(*action, None);
                    } else if let Some(shortcut) = parse_shortcut(value) {
                        bindings.insert(*action, Some(shortcut));
                    }
                }
            }
        }
        Self { bindings }
    }

    pub(crate) fn binding(&self, action: ShortcutAction) -> Option<KeyboardShortcut> {
        self.bindings.get(&action).copied().flatten()
    }

    /// Texte du raccourci pour les libellés d'interface, ex. « Ctrl+C ».
    pub(crate) fn binding_text(&self, action: ShortcutAction) -> Option<String> {
        self.binding(action).map(|s| format_shortcut(&s))
    }

    /// Libellé de menu : « Copier (Ctrl+C) », ou juste « Copier » sans raccourci.
    pub(crate) fn menu_label(&self, text: &str, action: ShortcutAction) -> String {
        match self.binding_text(action) {
            Some(binding) => format!("{text} ({binding})"),
            None => text.to_owned(),
        }
    }

    /// Assigne `shortcut` à `action`. Une autre action qui l'utilisait déjà
    /// perd son raccourci (pas de doublon silencieux). Ne sauvegarde pas :
    /// l'appelant enchaîne avec [`Self::save`].
    pub(crate) fn set(&mut self, action: ShortcutAction, shortcut: KeyboardShortcut) {
        for value in self.bindings.values_mut() {
            if *value == Some(shortcut) {
                *value = None;
            }
        }
        self.bindings.insert(action, Some(shortcut));
    }

    pub(crate) fn reset(&mut self) {
        self.bindings = Self::defaults();
    }

    pub(crate) fn save(&self) {
        let Some(path) = config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut content = String::new();
        for action in ShortcutAction::ALL {
            let value = self
                .binding(action)
                .map(|s| format_shortcut(&s))
                .unwrap_or_default();
            content.push_str(&format!("{}={value}\n", action.config_key()));
        }
        let _ = std::fs::write(path, content);
    }
}

fn config_path() -> Option<PathBuf> {
    crate::fs_ops::config_dir().map(|dir| dir.join("shortcuts.conf"))
}

pub(crate) fn format_shortcut(shortcut: &KeyboardShortcut) -> String {
    let mut parts = Vec::new();
    if shortcut.modifiers.ctrl || shortcut.modifiers.command {
        parts.push("Ctrl");
    }
    if shortcut.modifiers.shift {
        parts.push("Shift");
    }
    if shortcut.modifiers.alt {
        parts.push("Alt");
    }
    parts.push(shortcut.logical_key.name());
    parts.join("+")
}

fn parse_shortcut(text: &str) -> Option<KeyboardShortcut> {
    let mut modifiers = Modifiers::NONE;
    let mut key = None;
    for part in text.split('+') {
        match part.trim() {
            "" => continue,
            p if p.eq_ignore_ascii_case("ctrl") => modifiers = modifiers.plus(Modifiers::CTRL),
            p if p.eq_ignore_ascii_case("shift") => modifiers = modifiers.plus(Modifiers::SHIFT),
            p if p.eq_ignore_ascii_case("alt") => modifiers = modifiers.plus(Modifiers::ALT),
            p => key = Key::from_name(p),
        }
    }
    key.map(|k| KeyboardShortcut::new(modifiers, k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_format_roundtrip() {
        for action in ShortcutAction::ALL {
            let binding = action.default_binding();
            let text = format_shortcut(&binding);
            assert_eq!(parse_shortcut(&text), Some(binding), "roundtrip de {text}");
        }
        assert_eq!(parse_shortcut("n'importe quoi"), None);
    }

    #[test]
    fn set_steals_duplicate_binding() {
        let mut map = ShortcutMap {
            bindings: ShortcutMap::defaults(),
        };
        let ctrl_c = map.binding(ShortcutAction::Copy).unwrap();
        map.set(ShortcutAction::Cut, ctrl_c);
        assert_eq!(map.binding(ShortcutAction::Cut), Some(ctrl_c));
        assert_eq!(map.binding(ShortcutAction::Copy), None);
    }
}
