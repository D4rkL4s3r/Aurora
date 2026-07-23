use crate::actions::{Dialog, UiAction};
use crate::fs_ops::{self, QuickAccessEntry};
use crate::pane::Pane;
use crate::ui;
use std::path::PathBuf;

pub struct App {
    pub(crate) pane: Pane,
    pub(crate) drives: Vec<PathBuf>,
    pub(crate) quick_access: Vec<QuickAccessEntry>,
    pub(crate) clipboard: Vec<PathBuf>,
    pub(crate) clipboard_cut: bool,
    pub(crate) dialog: Option<Dialog>,
    pub(crate) status: Option<String>,
    pub(crate) dark_theme: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            pane: Pane::new(std::env::current_dir().unwrap_or_default()),
            drives: fs_ops::list_drives(),
            quick_access: fs_ops::list_quick_access(),
            clipboard: Vec::new(),
            clipboard_cut: false,
            dialog: None,
            status: None,
            dark_theme: true,
        }
    }
}

impl App {
    fn navigate_to(&mut self, path: PathBuf) {
        self.pane.navigate_to(path);
        self.status = None;
    }

    pub(crate) fn go_parent(&mut self) {
        self.pane.go_parent();
        self.status = None;
    }

    pub(crate) fn go_back(&mut self) {
        self.pane.go_back();
        self.status = None;
    }

    fn copy_selection(&mut self, cut: bool) {
        let selected = self.pane.selected_in_order();
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
                fs_ops::move_into(src, &self.pane.current_path)
            } else {
                fs_ops::copy_into(src, &self.pane.current_path)
            };
            if let Err(e) = result {
                errors.push(format!("{} : {e}", src.display()));
            }
        }
        if self.clipboard_cut {
            self.clipboard.clear();
            self.clipboard_cut = false;
        }
        self.pane.reload();
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
            UiAction::Select { path, ctrl, shift } => self.pane.select(path, ctrl, shift),
            UiAction::ContextSelect(path) => {
                if !self.pane.selection.contains(&path) {
                    self.pane.set_selection_to(path);
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
                let targets = self.pane.selected_in_order();
                if !targets.is_empty() {
                    self.dialog = Some(Dialog::ConfirmDelete { targets });
                }
            }
            UiAction::CopySelection { cut } => self.copy_selection(cut),
            UiAction::Paste => self.paste(),
            UiAction::SortBy(column) => self.pane.set_sort(column),
            UiAction::RunRecursiveSearch => {
                if let Some(status) = self.pane.run_recursive_search() {
                    self.status = Some(status);
                }
            }
        }
    }

    pub(crate) fn apply_dialog(&mut self, dialog: Dialog) {
        match dialog {
            Dialog::NewFolder { name } => {
                let name = name.trim();
                if name.is_empty() {
                    return;
                }
                match fs_ops::create_dir(&self.pane.current_path, name) {
                    Ok(_) => {
                        self.pane.reload();
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
                        self.pane.selection.remove(&target);
                        self.pane.set_selection_to(new_path);
                        self.pane.reload();
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
                self.pane.reload();
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
            self.pane.select_all();
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
            if let [single] = self.pane.selected_in_order().as_slice() {
                actions.push(UiAction::StartRename(single.clone()));
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.pane.poll_load();
        if self.pane.is_loading() {
            // Repeindre régulièrement tant que le thread de chargement travaille,
            // sinon le résultat n'est intégré qu'à la prochaine interaction.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }

        let mut actions: Vec<UiAction> = Vec::new();

        ui::toolbar::show(self, ui, &mut actions);
        ui::statusbar::show(self, ui);
        ui::sidebar::show(self, ui, &mut actions);
        ui::file_table::show(self, ui, &mut actions);

        self.handle_shortcuts(ui, &mut actions);

        for action in actions {
            self.apply_action(action);
        }

        ui::dialogs::show(self, ui.ctx());
    }
}
