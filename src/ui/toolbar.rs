use crate::actions::{Dialog, UiAction};
use crate::app::App;

pub(crate) fn show(app: &mut App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    egui::Panel::top("toolbar").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!app.pane.history.is_empty(), |ui| {
                if ui.button("⬅ Précédent").clicked() {
                    app.go_back();
                }
            });
            if ui.button("⬆ Dossier parent").clicked() {
                app.go_parent();
            }
            ui.separator();
            if ui.button("📁+ Nouveau dossier").clicked() {
                app.dialog = Some(Dialog::NewFolder {
                    name: String::new(),
                });
            }
            ui.add_enabled_ui(!app.clipboard.is_empty(), |ui| {
                if ui.button("📋 Coller").clicked() {
                    actions.push(UiAction::Paste);
                }
            });
            ui.separator();
            ui.label(app.pane.current_path.display().to_string());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !app.pane.search_query.is_empty() && ui.button("✖").clicked() {
                    app.pane.clear_search();
                    app.status = None;
                }
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut app.pane.search_query)
                        .desired_width(200.0)
                        .hint_text("🔍 Rechercher (Entrée : sous-dossiers)"),
                );
                if edit.changed() {
                    app.pane.search_results = None;
                }
                if edit.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !app.pane.search_query.trim().is_empty()
                {
                    actions.push(UiAction::RunRecursiveSearch);
                }
            });
        });
    });
}
