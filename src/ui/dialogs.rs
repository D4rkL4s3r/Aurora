use crate::actions::Dialog;
use crate::app::App;

pub(crate) fn show(app: &mut App, ctx: &egui::Context) {
    let Some(mut dialog) = app.dialog.take() else {
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
        app.apply_dialog(dialog);
    } else if !cancelled {
        app.dialog = Some(dialog);
    }
}
