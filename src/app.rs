use crate::fs_ops::{self, FileEntry, QuickAccessEntry};
use std::collections::HashSet;
use std::path::PathBuf;

enum Dialog {
    NewFolder { name: String },
    Rename { target: PathBuf, name: String },
    ConfirmDelete { targets: Vec<PathBuf> },
}

enum UiAction {
    Navigate(PathBuf),
    OpenFile(PathBuf),
    Select { path: PathBuf, ctrl: bool, shift: bool },
    ContextSelect(PathBuf),
    StartRename(PathBuf),
    AskDeleteSelection,
    CopySelection { cut: bool },
    Paste,
}

pub struct App {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    history: Vec<PathBuf>,
    drives: Vec<PathBuf>,
    quick_access: Vec<QuickAccessEntry>,
    selection: HashSet<PathBuf>,
    selection_anchor: Option<PathBuf>,
    clipboard: Vec<PathBuf>,
    clipboard_cut: bool,
    dialog: Option<Dialog>,
    status: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            current_path: std::env::current_dir().unwrap_or_default(),
            entries: Vec::new(),
            history: Vec::new(),
            drives: fs_ops::list_drives(),
            quick_access: fs_ops::list_quick_access(),
            selection: HashSet::new(),
            selection_anchor: None,
            clipboard: Vec::new(),
            clipboard_cut: false,
            dialog: None,
            status: None,
        };
        app.reload();
        app
    }
}

fn quick_access_icon(name: &str) -> &'static str {
    match name {
        "Bureau" => "🖥",
        "Documents" => "📄",
        "Téléchargements" => "⬇",
        "Images" => "🖼",
        "Musique" => "🎵",
        "Vidéos" => "🎬",
        _ => "⭐",
    }
}

impl App {
    fn reload(&mut self) {
        self.entries = fs_ops::list_dir(&self.current_path).unwrap_or_default();
        self.entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        let existing: HashSet<&PathBuf> = self.entries.iter().map(|e| &e.path).collect();
        self.selection.retain(|p| existing.contains(p));
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path != self.current_path {
            self.history.push(self.current_path.clone());
            self.current_path = path;
            self.selection.clear();
            self.selection_anchor = None;
            self.status = None;
            self.reload();
        }
    }

    fn go_parent(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.navigate_to(parent.to_path_buf());
        }
    }

    fn go_back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.current_path = previous;
            self.selection.clear();
            self.selection_anchor = None;
            self.status = None;
            self.reload();
        }
    }

    /// Chemins sélectionnés, dans l'ordre d'affichage.
    fn selected_in_order(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter(|e| self.selection.contains(&e.path))
            .map(|e| e.path.clone())
            .collect()
    }

    fn select(&mut self, path: PathBuf, ctrl: bool, shift: bool) {
        if shift {
            let anchor = self
                .selection_anchor
                .as_ref()
                .and_then(|a| self.entries.iter().position(|e| &e.path == a));
            let clicked = self.entries.iter().position(|e| e.path == path);
            if let (Some(anchor), Some(clicked)) = (anchor, clicked) {
                let (from, to) = (anchor.min(clicked), anchor.max(clicked));
                if !ctrl {
                    self.selection.clear();
                }
                for entry in &self.entries[from..=to] {
                    self.selection.insert(entry.path.clone());
                }
                return; // l'ancre ne bouge pas : Shift+clic successifs étendent depuis la même origine
            }
        }
        if ctrl {
            if !self.selection.remove(&path) {
                self.selection.insert(path.clone());
            }
        } else {
            self.selection.clear();
            self.selection.insert(path.clone());
        }
        self.selection_anchor = Some(path);
    }

    fn select_all(&mut self) {
        self.selection = self.entries.iter().map(|e| e.path.clone()).collect();
    }

    fn copy_selection(&mut self, cut: bool) {
        let selected = self.selected_in_order();
        if !selected.is_empty() {
            self.clipboard = selected;
            self.clipboard_cut = cut;
            let verb = if cut { "à déplacer" } else { "à copier" };
            self.status = Some(format!("{} élément(s) {verb}", self.clipboard.len()));
        }
    }

    fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        let items = self.clipboard.clone();
        let mut errors = Vec::new();
        for src in &items {
            let result = if self.clipboard_cut {
                fs_ops::move_into(src, &self.current_path)
            } else {
                fs_ops::copy_into(src, &self.current_path)
            };
            if let Err(e) = result {
                errors.push(format!("{} : {e}", src.display()));
            }
        }
        if self.clipboard_cut {
            self.clipboard.clear();
            self.clipboard_cut = false;
        }
        self.reload();
        self.status = if errors.is_empty() {
            Some(format!("{} élément(s) collé(s)", items.len()))
        } else {
            Some(errors.join(" · "))
        };
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::Navigate(path) => self.navigate_to(path),
            UiAction::OpenFile(path) => {
                if let Err(e) = fs_ops::open_with_default_app(&path) {
                    self.status = Some(format!("Ouverture impossible : {e}"));
                }
            }
            UiAction::Select { path, ctrl, shift } => self.select(path, ctrl, shift),
            UiAction::ContextSelect(path) => {
                if !self.selection.contains(&path) {
                    self.selection.clear();
                    self.selection.insert(path.clone());
                    self.selection_anchor = Some(path);
                }
            }
            UiAction::StartRename(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.dialog = Some(Dialog::Rename { target: path, name });
            }
            UiAction::AskDeleteSelection => {
                let targets = self.selected_in_order();
                if !targets.is_empty() {
                    self.dialog = Some(Dialog::ConfirmDelete { targets });
                }
            }
            UiAction::CopySelection { cut } => self.copy_selection(cut),
            UiAction::Paste => self.paste(),
        }
    }

    fn apply_dialog(&mut self, dialog: Dialog) {
        match dialog {
            Dialog::NewFolder { name } => {
                let name = name.trim();
                if name.is_empty() {
                    return;
                }
                match fs_ops::create_dir(&self.current_path, name) {
                    Ok(_) => {
                        self.reload();
                        self.status = Some(format!("Dossier « {name} » créé"));
                    }
                    Err(e) => self.status = Some(format!("Création impossible : {e}")),
                }
            }
            Dialog::Rename { target, name } => {
                let name = name.trim();
                if name.is_empty() {
                    return;
                }
                match fs_ops::rename_entry(&target, name) {
                    Ok(new_path) => {
                        self.selection.remove(&target);
                        self.selection.insert(new_path.clone());
                        self.selection_anchor = Some(new_path);
                        self.reload();
                    }
                    Err(e) => self.status = Some(format!("Renommage impossible : {e}")),
                }
            }
            Dialog::ConfirmDelete { targets } => {
                match fs_ops::delete_to_trash(&targets) {
                    Ok(()) => {
                        self.status =
                            Some(format!("{} élément(s) envoyé(s) à la corbeille", targets.len()));
                    }
                    Err(e) => self.status = Some(format!("Suppression impossible : {e}")),
                }
                self.reload();
            }
        }
    }

    fn handle_shortcuts(&mut self, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
        if self.dialog.is_some() || ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        use egui::{Key, KeyboardShortcut, Modifiers};
        let ctrl = Modifiers::CTRL;
        if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::A))) {
            self.select_all();
        }
        if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::C))) {
            actions.push(UiAction::CopySelection { cut: false });
        }
        if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::X))) {
            actions.push(UiAction::CopySelection { cut: true });
        }
        if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(ctrl, Key::V))) {
            actions.push(UiAction::Paste);
        }
        if ui.input(|i| i.key_pressed(Key::Delete)) {
            actions.push(UiAction::AskDeleteSelection);
        }
        if ui.input(|i| i.key_pressed(Key::F2)) {
            if let [single] = self.selected_in_order().as_slice() {
                actions.push(UiAction::StartRename(single.clone()));
            }
        }
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        let mut confirmed = false;
        let mut cancelled = false;

        let modal = egui::Modal::new(egui::Id::new("aurora_dialog")).show(ctx, |ui| {
            ui.set_width(300.0);
            match &mut dialog {
                Dialog::NewFolder { name } => {
                    ui.heading("Nouveau dossier");
                    ui.add_space(8.0);
                    let edit = ui.text_edit_singleline(name);
                    edit.request_focus();
                    if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        confirmed = true;
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Créer").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Annuler").clicked() {
                            cancelled = true;
                        }
                    });
                }
                Dialog::Rename { name, .. } => {
                    ui.heading("Renommer");
                    ui.add_space(8.0);
                    let edit = ui.text_edit_singleline(name);
                    edit.request_focus();
                    if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        confirmed = true;
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Renommer").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Annuler").clicked() {
                            cancelled = true;
                        }
                    });
                }
                Dialog::ConfirmDelete { targets } => {
                    ui.heading("Supprimer");
                    ui.add_space(8.0);
                    if let [single] = targets.as_slice() {
                        let name = single
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| single.display().to_string());
                        ui.label(format!("Envoyer « {name} » à la corbeille ?"));
                    } else {
                        ui.label(format!(
                            "Envoyer ces {} éléments à la corbeille ?",
                            targets.len()
                        ));
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        confirmed = true;
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Supprimer").clicked() {
                            confirmed = true;
                        }
                        if ui.button("Annuler").clicked() {
                            cancelled = true;
                        }
                    });
                }
            }
        });

        if modal.should_close() {
            cancelled = true;
        }
        if confirmed {
            self.apply_dialog(dialog);
        } else if !cancelled {
            self.dialog = Some(dialog);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut actions: Vec<UiAction> = Vec::new();

        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(!self.history.is_empty(), |ui| {
                    if ui.button("⬅ Précédent").clicked() {
                        self.go_back();
                    }
                });
                if ui.button("⬆ Dossier parent").clicked() {
                    self.go_parent();
                }
                ui.separator();
                if ui.button("📁+ Nouveau dossier").clicked() {
                    self.dialog = Some(Dialog::NewFolder {
                        name: String::new(),
                    });
                }
                ui.add_enabled_ui(!self.clipboard.is_empty(), |ui| {
                    if ui.button("📋 Coller").clicked() {
                        actions.push(UiAction::Paste);
                    }
                });
                ui.separator();
                ui.label(self.current_path.display().to_string());
            });
        });

        egui::Panel::bottom("statusbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} élément(s) · {} sélectionné(s)",
                    self.entries.len(),
                    self.selection.len()
                ));
                if let Some(status) = &self.status {
                    ui.separator();
                    ui.label(status.clone());
                }
            });
        });

        egui::Panel::left("sidebar").show(ui, |ui| {
            ui.label("Accès rapide");
            ui.separator();
            for quick_access in &self.quick_access {
                let is_current = self.current_path.starts_with(&quick_access.path);
                let icon = quick_access_icon(quick_access.name);
                let label = format!("{icon} {}", quick_access.name);
                if ui.selectable_label(is_current, label).clicked() {
                    actions.push(UiAction::Navigate(quick_access.path.clone()));
                }
            }

            ui.add_space(8.0);
            ui.label("Lecteurs");
            ui.separator();
            for drive in &self.drives {
                let is_current = self.current_path.starts_with(drive);
                let label = drive.display().to_string();
                if ui.selectable_label(is_current, format!("💾 {label}")).clicked() {
                    actions.push(UiAction::Navigate(drive.clone()));
                }
            }
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.entries {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let is_selected = self.selection.contains(&entry.path);
                let is_cut = self.clipboard_cut && self.clipboard.contains(&entry.path);
                let mut text = egui::RichText::new(format!("{icon} {}", entry.name));
                if is_cut {
                    text = text.weak();
                }
                let response = ui.selectable_label(is_selected, text);

                if response.clicked() {
                    let modifiers = ui.input(|i| i.modifiers);
                    actions.push(UiAction::Select {
                        path: entry.path.clone(),
                        ctrl: modifiers.ctrl,
                        shift: modifiers.shift,
                    });
                }
                if response.double_clicked() {
                    if entry.is_dir {
                        actions.push(UiAction::Navigate(entry.path.clone()));
                    } else {
                        actions.push(UiAction::OpenFile(entry.path.clone()));
                    }
                }
                if response.secondary_clicked() {
                    actions.push(UiAction::ContextSelect(entry.path.clone()));
                }
                response.context_menu(|ui| {
                    if !entry.is_dir && ui.button("Ouvrir").clicked() {
                        actions.push(UiAction::OpenFile(entry.path.clone()));
                        ui.close();
                    }
                    if ui.button("Renommer (F2)").clicked() {
                        actions.push(UiAction::StartRename(entry.path.clone()));
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Copier (Ctrl+C)").clicked() {
                        actions.push(UiAction::CopySelection { cut: false });
                        ui.close();
                    }
                    if ui.button("Couper (Ctrl+X)").clicked() {
                        actions.push(UiAction::CopySelection { cut: true });
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Supprimer (Suppr)").clicked() {
                        actions.push(UiAction::AskDeleteSelection);
                        ui.close();
                    }
                });
            }
        });

        self.handle_shortcuts(ui, &mut actions);

        for action in actions {
            self.apply_action(action);
        }

        self.show_dialog(ui.ctx());
    }
}
