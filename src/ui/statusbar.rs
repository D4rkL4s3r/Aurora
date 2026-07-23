use crate::app::App;

pub(crate) fn show(app: &App, ui: &mut egui::Ui) {
    let frame =
        egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(10, 5));
    egui::Panel::bottom("statusbar").frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} élément(s) · {} sélectionné(s)",
                    app.pane.displayed_count(),
                    app.pane.selection.len()
                ))
                .small()
                .weak(),
            );
            if let Some(status) = &app.status {
                ui.separator();
                ui.label(egui::RichText::new(status.clone()).small());
            }
        });
    });
}
