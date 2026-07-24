use crate::actions::{Dialog, DragPayload, UiAction};
use crate::fs_ops::{self, QuickAccessEntry};
use crate::pane::Pane;
use crate::shortcuts::{ShortcutAction, ShortcutMap, ShortcutsEditor};
use crate::tab::Tab;
use crate::ui;
use std::path::PathBuf;

pub struct App {
    /// Onglets ouverts (toujours au moins un).
    pub(crate) tabs: Vec<Tab>,
    /// Index de l'onglet actif, cible de la toolbar, de la sidebar et des raccourcis.
    pub(crate) active_tab: usize,
    pub(crate) drives: Vec<PathBuf>,
    pub(crate) quick_access: Vec<QuickAccessEntry>,
    pub(crate) clipboard: Vec<PathBuf>,
    pub(crate) clipboard_cut: bool,
    pub(crate) dialog: Option<Dialog>,
    pub(crate) status: Option<String>,
    pub(crate) dark_theme: bool,
    pub(crate) grid_view: bool,
    /// RefCell : le rendu (en `&App`) alimente le cache d'icônes au fil des frames.
    pub(crate) icons: std::cell::RefCell<ui::icons::IconCache>,
    pub(crate) shortcuts: ShortcutMap,
    /// Fenêtre de configuration des raccourcis, si ouverte.
    pub(crate) shortcuts_editor: Option<ShortcutsEditor>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::new(std::env::current_dir().unwrap_or_default())],
            active_tab: 0,
            drives: fs_ops::list_drives(),
            quick_access: fs_ops::list_quick_access(),
            clipboard: Vec::new(),
            clipboard_cut: false,
            dialog: None,
            status: None,
            dark_theme: true,
            grid_view: true,
            icons: std::cell::RefCell::new(ui::icons::IconCache::default()),
            shortcuts: ShortcutMap::load(),
            shortcuts_editor: None,
        }
    }
}

impl App {
    pub(crate) fn tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub(crate) fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub(crate) fn pane(&self) -> &Pane {
        self.tab().pane()
    }

    pub(crate) fn pane_mut(&mut self) -> &mut Pane {
        self.tab_mut().pane_mut()
    }

    pub(crate) fn is_split(&self) -> bool {
        self.tab().is_split()
    }

    pub(crate) fn toggle_split(&mut self) {
        self.tab_mut().toggle_split();
    }

    pub(crate) fn new_tab(&mut self) {
        let path = self.pane().current_path.clone();
        self.tabs.push(Tab::new(path));
        self.active_tab = self.tabs.len() - 1;
    }

    pub(crate) fn close_tab(&mut self, idx: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(idx);
        if self.active_tab >= idx && self.active_tab > 0 {
            self.active_tab -= 1;
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
    }

    pub(crate) fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    /// Recharge tous les volets de tous les onglets : après une opération
    /// fichier, chacun peut afficher le dossier modifié.
    fn reload_panes(&mut self) {
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                pane.reload();
            }
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
        self.pane_mut().navigate_to(path);
        self.status = None;
    }

    pub(crate) fn go_parent(&mut self) {
        self.pane_mut().go_parent();
        self.status = None;
    }

    pub(crate) fn go_back(&mut self) {
        self.pane_mut().go_back();
        self.status = None;
    }

    fn copy_selection(&mut self, cut: bool) {
        let selected = self.pane().selected_in_order();
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
        let dest = self.pane().current_path.clone();
        let mut errors = Vec::new();
        for src in &items {
            let result = if self.clipboard_cut {
                fs_ops::move_into(src, &dest)
            } else {
                fs_ops::copy_into(src, &dest)
            };
            if let Err(e) = result {
                errors.push(format!("{} : {e}", src.display()));
            }
        }
        if self.clipboard_cut {
            self.clipboard.clear();
            self.clipboard_cut = false;
        }
        self.reload_panes();
        self.status = if errors.is_empty() {
            Some(format!("{} élément(s) collé(s)", items.len()))
        } else {
            Some(errors.join(" · "))
        };
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::FocusPane(idx) => {
                let tab = self.tab_mut();
                tab.active_pane = idx.min(tab.panes.len().saturating_sub(1));
            }
            UiAction::Navigate(path) => self.navigate_to(path),
            UiAction::OpenFile(path) => {
                if let Err(e) = fs_ops::open_with_default_app(&path) {
                    self.status = Some(format!("Ouverture impossible : {e}"));
                }
            }
            UiAction::Select { path, ctrl, shift } => self.pane_mut().select(path, ctrl, shift),
            UiAction::ContextSelect(path) => {
                if !self.pane().selection.contains(&path) {
                    self.pane_mut().set_selection_to(path);
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
                let targets = self.pane().selected_in_order();
                if !targets.is_empty() {
                    self.dialog = Some(Dialog::ConfirmDelete { targets });
                }
            }
            UiAction::CopySelection { cut } => self.copy_selection(cut),
            UiAction::Paste => self.paste(),
            UiAction::SortBy(column) => self.pane_mut().set_sort(column),
            UiAction::DropPaths { paths, dest, copy } => self.drop_paths(paths, dest, copy),
            UiAction::RunRecursiveSearch => {
                if let Some(status) = self.pane_mut().run_recursive_search() {
                    self.status = Some(status);
                }
            }
            UiAction::OpenTerminal => {
                if let Err(e) = fs_ops::open_terminal_here(&self.pane().current_path) {
                    self.status = Some(format!("Terminal impossible : {e}"));
                }
            }
            UiAction::OpenVsCode => {
                if let Err(e) = fs_ops::open_in_vscode(&self.pane().current_path) {
                    self.status = Some(format!("VS Code impossible : {e}"));
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
                match fs_ops::create_dir(&self.pane().current_path, name) {
                    Ok(_) => {
                        self.reload_panes();
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
                        let pane = self.pane_mut();
                        pane.selection.remove(&target);
                        pane.set_selection_to(new_path);
                        self.reload_panes();
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
                self.reload_panes();
            }
        }
    }

    /// Applique un glisser-déposer interne : déplace (ou copie avec Ctrl)
    /// les chemins traînés dans `dest`.
    fn drop_paths(&mut self, paths: Vec<PathBuf>, dest: PathBuf, copy: bool) {
        let paths: Vec<PathBuf> = if copy {
            paths
        } else {
            // Déplacer un élément là où il est déjà est un non-événement.
            paths
                .into_iter()
                .filter(|p| p.parent() != Some(dest.as_path()))
                .collect()
        };
        if paths.is_empty() {
            return;
        }
        let mut errors = Vec::new();
        for src in &paths {
            let result = if copy {
                fs_ops::copy_into(src, &dest)
            } else {
                fs_ops::move_into(src, &dest)
            };
            if let Err(e) = result {
                errors.push(format!("{} : {e}", src.display()));
            }
        }
        self.reload_panes();
        self.status = if errors.is_empty() {
            let verb = if copy { "copié(s)" } else { "déplacé(s)" };
            Some(format!(
                "{} élément(s) {verb} vers {}",
                paths.len(),
                dest.display()
            ))
        } else {
            Some(errors.join(" · "))
        };
    }

    /// Étiquette qui suit le curseur pendant un glisser-déposer interne.
    fn draw_drag_ghost(&self, ctx: &egui::Context) {
        let Some(payload) = egui::DragAndDrop::payload::<DragPayload>(ctx) else {
            return;
        };
        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        let Some(pos) = ctx.pointer_latest_pos() else {
            return;
        };
        let copy = ctx.input(|i| i.modifiers.ctrl);
        let verb = if copy { "copier" } else { "déplacer" };
        let text = format!("{} élément(s) à {verb}", payload.paths.len());
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("drag_ghost"),
        ));
        let galley = painter.layout_no_wrap(
            text,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
        let rect = egui::Rect::from_min_size(
            pos + egui::vec2(14.0, 10.0),
            galley.size() + egui::vec2(12.0, 8.0),
        );
        painter.rect_filled(rect, 6.0, crate::ui::theme::ACCENT.gamma_multiply(0.85));
        painter.galley(rect.min + egui::vec2(6.0, 4.0), galley, egui::Color32::WHITE);
    }

    /// Glisser-déposer natif : fichiers déposés depuis l'Explorateur Windows
    /// → copie dans le dossier courant, avec un voile indicatif pendant le survol.
    fn handle_file_drops(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("drop_overlay"),
            ));
            let rect = ctx.content_rect();
            painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(110));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("Déposer pour copier dans {}", self.pane().current_path.display()),
                egui::FontId::proportional(18.0),
                egui::Color32::WHITE,
            );
        }

        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if dropped.is_empty() {
            return;
        }
        let dest = self.pane().current_path.clone();
        let mut errors = Vec::new();
        for src in &dropped {
            if let Err(e) = fs_ops::copy_into(src, &dest) {
                errors.push(format!("{} : {e}", src.display()));
            }
        }
        self.reload_panes();
        self.status = if errors.is_empty() {
            Some(format!("{} élément(s) copié(s)", dropped.len()))
        } else {
            Some(errors.join(" · "))
        };
    }

    fn handle_shortcuts(&mut self, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
        if self.dialog.is_some()
            || self.shortcuts_editor.is_some()
            || ui.ctx().egui_wants_keyboard_input()
        {
            return;
        }
        let triggered: Vec<ShortcutAction> = ui.input_mut(|i| {
            ShortcutAction::ALL
                .iter()
                .copied()
                .filter(|a| {
                    self.shortcuts
                        .binding(*a)
                        .is_some_and(|s| i.consume_shortcut(&s))
                })
                .collect()
        });
        for action in triggered {
            match action {
                ShortcutAction::SelectAll => self.pane_mut().select_all(),
                ShortcutAction::Copy => actions.push(UiAction::CopySelection { cut: false }),
                ShortcutAction::Cut => actions.push(UiAction::CopySelection { cut: true }),
                ShortcutAction::Paste => actions.push(UiAction::Paste),
                ShortcutAction::Delete => actions.push(UiAction::AskDeleteSelection),
                ShortcutAction::Rename => {
                    if let [single] = self.pane().selected_in_order().as_slice() {
                        actions.push(UiAction::StartRename(single.clone()));
                    }
                }
                ShortcutAction::AddressBar => self.pane_mut().open_address_bar(),
                ShortcutAction::ToggleSplit => self.toggle_split(),
                ShortcutAction::NewTab => self.new_tab(),
                ShortcutAction::CloseTab => self.close_tab(self.active_tab),
                ShortcutAction::NextTab => self.next_tab(),
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                pane.poll_load();
            }
        }
        if self
            .tabs
            .iter()
            .flat_map(|t| t.panes.iter())
            .any(Pane::is_loading)
        {
            // Repeindre régulièrement tant qu'un thread de chargement travaille,
            // sinon le résultat n'est intégré qu'à la prochaine interaction.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }

        let mut actions: Vec<UiAction> = Vec::new();

        ui::tabs::show(self, ui);
        ui::toolbar::show(self, ui, &mut actions);
        ui::statusbar::show(self, ui, &mut actions);
        ui::sidebar::show(self, ui, &mut actions);
        ui::file_table::show(self, ui, &mut actions);

        self.handle_shortcuts(ui, &mut actions);
        self.handle_file_drops(ui.ctx());
        self.draw_drag_ghost(ui.ctx());

        for action in actions {
            self.apply_action(action);
        }

        ui::dialogs::show(self, ui.ctx());
        ui::settings::show(self, ui.ctx());
    }
}
