use crate::fs_ops::{self, FileEntry};
use std::path::PathBuf;

pub struct App {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    history: Vec<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            current_path: std::env::current_dir().unwrap_or_default(),
            entries: Vec::new(),
            history: Vec::new(),
        };
        app.reload();
        app
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
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path != self.current_path {
            self.history.push(self.current_path.clone());
            self.current_path = path;
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
            self.reload();
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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
                ui.label(self.current_path.display().to_string());
            });
        });

        let mut to_navigate = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.entries {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let response = ui.selectable_label(false, format!("{icon} {}", entry.name));
                if response.double_clicked() && entry.is_dir {
                    to_navigate = Some(entry.path.clone());
                }
            }
        });

        if let Some(path) = to_navigate {
            self.navigate_to(path);
        }
    }
}
