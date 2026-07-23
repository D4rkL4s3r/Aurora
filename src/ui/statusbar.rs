use crate::app::App;

pub(crate) fn show(app: &App, ui: &mut egui::Ui) {
    egui::Panel::bottom("statusbar").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} élément(s) · {} sélectionné(s)",
                app.pane.displayed_count(),
                app.pane.selection.len()
            ));
            if let Some(status) = &app.status {
                ui.separator();
                ui.label(status.clone());
            }
        });
    });
}
