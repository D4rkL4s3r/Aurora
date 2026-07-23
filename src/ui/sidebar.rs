use crate::actions::UiAction;
use crate::app::App;
use crate::ui::format::quick_access_icon;

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(title).small().weak());
    ui.add_space(2.0);
}

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let frame =
        egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(10, 8));
    egui::Panel::left("sidebar").frame(frame).show(ui, |ui| {
        // Items étirés sur toute la largeur : zone de clic et surbrillance pleine ligne
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            section_header(ui, "ACCÈS RAPIDE");
            for quick_access in &app.quick_access {
                let is_current = app.pane.current_path.starts_with(&quick_access.path);
                let icon = quick_access_icon(quick_access.name);
                let label = format!("{icon} {}", quick_access.name);
                if ui.selectable_label(is_current, label).clicked() {
                    actions.push(UiAction::Navigate(quick_access.path.clone()));
                }
            }

            ui.add_space(10.0);
            section_header(ui, "LECTEURS");
            for drive in &app.drives {
                let is_current = app.pane.current_path.starts_with(drive);
                let label = drive.display().to_string();
                if ui.selectable_label(is_current, format!("💾 {label}")).clicked() {
                    actions.push(UiAction::Navigate(drive.clone()));
                }
            }
        });
    });
}
