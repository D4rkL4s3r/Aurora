use crate::actions::UiAction;
use crate::app::App;
use crate::ui::format::quick_access_icon;

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    egui::Panel::left("sidebar").show(ui, |ui| {
        ui.label("Accès rapide");
        ui.separator();
        for quick_access in &app.quick_access {
            let is_current = app.pane.current_path.starts_with(&quick_access.path);
            let icon = quick_access_icon(quick_access.name);
            let label = format!("{icon} {}", quick_access.name);
            if ui.selectable_label(is_current, label).clicked() {
                actions.push(UiAction::Navigate(quick_access.path.clone()));
            }
        }

        ui.add_space(8.0);
        ui.label("Lecteurs");
        ui.separator();
        for drive in &app.drives {
            let is_current = app.pane.current_path.starts_with(drive);
            let label = drive.display().to_string();
            if ui.selectable_label(is_current, format!("💾 {label}")).clicked() {
                actions.push(UiAction::Navigate(drive.clone()));
            }
        }
    });
}
