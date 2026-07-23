use crate::fs_ops::{self, FileEntry, QuickAccessEntry};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

enum Dialog {
    NewFolder { name: String },
    Rename { target: PathBuf, name: String },
    ConfirmDelete { targets: Vec<PathBuf> },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
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
    SortBy(SortColumn),
    RunRecursiveSearch,
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
    search_query: String,
    search_results: Option<Vec<FileEntry>>,
    sort_column: SortColumn,
    sort_ascending: bool,
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
            search_query: String::new(),
            search_results: None,
            sort_column: SortColumn::Name,
            sort_ascending: true,
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

fn entry_icon(entry: &FileEntry) -> &'static str {
    if entry.is_dir {
        return "📁";
    }
    let ext = entry.extension.as_deref().map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some(
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tif" | "tiff",
        ) => "🖼",
        Some("mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" | "m4v") => "🎬",
        Some("mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus") => "🎵",
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab") => "🗜",
        Some(
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "c" | "cpp" | "h" | "hpp" | "java"
            | "cs" | "go" | "rb" | "php" | "html" | "css" | "json" | "toml" | "yaml" | "yml"
            | "xml" | "sh" | "ps1" | "bat" | "cmd" | "sql" | "md",
        ) => "📝",
        Some("exe" | "msi" | "lnk" | "dll") => "⚙",
        Some("pdf") => "📕",
        Some("xls" | "xlsx" | "ods" | "csv") => "📊",
        Some("ppt" | "pptx" | "odp") => "📽",
        _ => "📄",
    }
}

fn type_label(entry: &FileEntry) -> String {
    if entry.is_dir {
        "Dossier".to_owned()
    } else {
        match &entry.extension {
            Some(ext) => format!("Fichier {}", ext.to_uppercase()),
            None => "Fichier".to_owned(),
        }
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["o", "Ko", "Mo", "Go", "To"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} o")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_date(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%d/%m/%Y %H:%M").to_string()
}

impl App {
    fn sort_entries(entries: &mut [FileEntry], column: SortColumn, ascending: bool) {
        entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then_with(|| {
                let ord = match column {
                    SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortColumn::Size => a.size.cmp(&b.size),
                    SortColumn::Type => {
                        let ext_a = a.extension.as_deref().unwrap_or("").to_lowercase();
                        let ext_b = b.extension.as_deref().unwrap_or("").to_lowercase();
                        ext_a
                            .cmp(&ext_b)
                            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    }
                    SortColumn::Modified => a.modified.cmp(&b.modified),
                };
                if ascending { ord } else { ord.reverse() }
            })
        });
    }

    fn reload(&mut self) {
        self.entries = fs_ops::list_dir(&self.current_path).unwrap_or_default();
        Self::sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
        if let Some(results) = &mut self.search_results {
            results.retain(|e| e.path.exists());
        }
        let mut existing: HashSet<&PathBuf> = self.entries.iter().map(|e| &e.path).collect();
        if let Some(results) = &self.search_results {
            existing.extend(results.iter().map(|e| &e.path));
        }
        self.selection.retain(|p| existing.contains(p));
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path != self.current_path {
            self.history.push(self.current_path.clone());
            self.current_path = path;
            self.selection.clear();
            self.selection_anchor = None;
            self.status = None;
            self.search_query.clear();
            self.search_results = None;
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
            self.search_query.clear();
            self.search_results = None;
            self.reload();
        }
    }

    /// Liste actuellement affichée : résultats de recherche récursive,
    /// ou contenu du dossier (filtré par la recherche instantanée).
    fn displayed_base(&self) -> &[FileEntry] {
        match &self.search_results {
            Some(results) => results,
            None => &self.entries,
        }
    }

    fn instant_filter(&self) -> Option<String> {
        let query = self.search_query.trim();
        (self.search_results.is_none() && !query.is_empty()).then(|| query.to_lowercase())
    }

    /// Chemins affichés, dans l'ordre d'affichage.
    fn displayed_paths(&self) -> Vec<PathBuf> {
        let base = self.displayed_base();
        match self.instant_filter() {
            Some(query) => base
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .map(|e| e.path.clone())
                .collect(),
            None => base.iter().map(|e| e.path.clone()).collect(),
        }
    }

    fn displayed_count(&self) -> usize {
        match self.instant_filter() {
            Some(query) => self
                .displayed_base()
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .count(),
            None => self.displayed_base().len(),
        }
    }

    /// Chemins sélectionnés, dans l'ordre d'affichage.
    fn selected_in_order(&self) -> Vec<PathBuf> {
        self.displayed_paths()
            .into_iter()
            .filter(|p| self.selection.contains(p))
            .collect()
    }

    fn select(&mut self, path: PathBuf, ctrl: bool, shift: bool) {
        let displayed = self.displayed_paths();
        if shift {
            let anchor = self
                .selection_anchor
                .as_ref()
                .and_then(|a| displayed.iter().position(|p| p == a));
            let clicked = displayed.iter().position(|p| p == &path);
            if let (Some(anchor), Some(clicked)) = (anchor, clicked) {
                let (from, to) = (anchor.min(clicked), anchor.max(clicked));
                if !ctrl {
                    self.selection.clear();
                }
                for p in &displayed[from..=to] {
                    self.selection.insert(p.clone());
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
        self.selection = self.displayed_paths().into_iter().collect();
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

    fn run_recursive_search(&mut self) {
        const MAX_DEPTH: usize = 8;
        const MAX_RESULTS: usize = 300;
        let query = self.search_query.trim().to_owned();
        if query.is_empty() {
            return;
        }
        let mut results =
            fs_ops::search_recursive(&self.current_path, &query, MAX_DEPTH, MAX_RESULTS);
        Self::sort_entries(&mut results, self.sort_column, self.sort_ascending);
        let capped = if results.len() >= MAX_RESULTS {
            " (limité à 300)"
        } else {
            ""
        };
        self.status = Some(format!(
            "{} résultat(s) dans les sous-dossiers{capped}",
            results.len()
        ));
        self.search_results = Some(results);
        self.selection.clear();
        self.selection_anchor = None;
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
            UiAction::SortBy(column) => {
                if self.sort_column == column {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = column;
                    self.sort_ascending = true;
                }
                Self::sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
                if let Some(results) = &mut self.search_results {
                    Self::sort_entries(results, self.sort_column, self.sort_ascending);
                }
            }
            UiAction::RunRecursiveSearch => self.run_recursive_search(),
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

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !self.search_query.is_empty() && ui.button("✖").clicked() {
                        self.search_query.clear();
                        self.search_results = None;
                        self.status = None;
                    }
                    let edit = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .desired_width(200.0)
                            .hint_text("🔍 Rechercher (Entrée : sous-dossiers)"),
                    );
                    if edit.changed() {
                        self.search_results = None;
                    }
                    if edit.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.search_query.trim().is_empty()
                    {
                        actions.push(UiAction::RunRecursiveSearch);
                    }
                });
            });
        });

        egui::Panel::bottom("statusbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} élément(s) · {} sélectionné(s)",
                    self.displayed_count(),
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

        {
            let base = self.displayed_base();
            let displayed_idx: Vec<usize> = match self.instant_filter() {
                Some(query) => base
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.name.to_lowercase().contains(&query))
                    .map(|(i, _)| i)
                    .collect(),
                None => (0..base.len()).collect(),
            };
            let is_search = self.search_results.is_some();
            let sort_label = |label: &str, column: SortColumn| -> String {
                if self.sort_column == column {
                    format!("{label} {}", if self.sort_ascending { "⬆" } else { "⬇" })
                } else {
                    label.to_owned()
                }
            };

            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .sense(egui::Sense::click())
                .column(egui_extras::Column::remainder().clip(true))
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto())
                .header(22.0, |mut header| {
                    header.col(|ui| {
                        if ui.selectable_label(false, sort_label("Nom", SortColumn::Name)).clicked()
                        {
                            actions.push(UiAction::SortBy(SortColumn::Name));
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(false, sort_label("Taille", SortColumn::Size))
                            .clicked()
                        {
                            actions.push(UiAction::SortBy(SortColumn::Size));
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(false, sort_label("Type", SortColumn::Type))
                            .clicked()
                        {
                            actions.push(UiAction::SortBy(SortColumn::Type));
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(false, sort_label("Modifié", SortColumn::Modified))
                            .clicked()
                        {
                            actions.push(UiAction::SortBy(SortColumn::Modified));
                        }
                    });
                })
                .body(|body| {
                    body.rows(20.0, displayed_idx.len(), |mut row| {
                        let entry = &base[displayed_idx[row.index()]];
                        let is_selected = self.selection.contains(&entry.path);
                        row.set_selected(is_selected);

                        let display_name = if is_search {
                            entry
                                .path
                                .strip_prefix(&self.current_path)
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|_| entry.name.clone())
                        } else {
                            entry.name.clone()
                        };
                        let is_cut =
                            self.clipboard_cut && self.clipboard.contains(&entry.path);
                        let mut text =
                            egui::RichText::new(format!("{} {display_name}", entry_icon(entry)));
                        if is_cut {
                            text = text.weak();
                        }

                        row.col(|ui| {
                            ui.label(text.clone());
                        });
                        row.col(|ui| {
                            if entry.is_dir {
                                ui.label("");
                            } else {
                                ui.label(format_size(entry.size));
                            }
                        });
                        row.col(|ui| {
                            ui.label(type_label(entry));
                        });
                        row.col(|ui| {
                            ui.label(entry.modified.map(format_date).unwrap_or_default());
                        });

                        let response = row.response();
                        if response.clicked() {
                            let modifiers = response.ctx.input(|i| i.modifiers);
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
                    });
                });
        }

        self.handle_shortcuts(ui, &mut actions);

        for action in actions {
            self.apply_action(action);
        }

        self.show_dialog(ui.ctx());
    }
}
