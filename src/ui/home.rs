//! Écran d'accueil d'un volet sans dossier : tuiles cliquables vers les
//! accès rapides et les lecteurs, pour choisir où commencer.

use crate::actions::UiAction;
use crate::app::App;
use crate::ui::format::quick_access_icon;

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(14.0);
    ui.label(egui::RichText::new(title).small().weak());
    ui.add_space(4.0);
}

fn tile(ui: &mut egui::Ui, text: String) -> bool {
    ui.add_sized(
        [180.0, 46.0],
        egui::Button::new(egui::RichText::new(text).size(14.0)),
    )
    .clicked()
}

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(28.0);
            ui.vertical_centered(|ui| {
                ui.heading("Accueil");
                ui.label(
                    egui::RichText::new("Choisissez un emplacement pour commencer").weak(),
                );
            });
            ui.add_space(10.0);

            section(ui, "ACCÈS RAPIDE");
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(10.0, 10.0);
                for quick_access in &app.quick_access {
                    let icon = quick_access_icon(quick_access.name);
                    if tile(ui, format!("{icon}  {}", quick_access.name)) {
                        actions.push(UiAction::Navigate(quick_access.path.clone()));
                    }
                }
            });

            section(ui, "LECTEURS");
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(10.0, 10.0);
                for drive in &app.drives {
                    if tile(ui, format!("💾  {}", drive.display())) {
                        actions.push(UiAction::Navigate(drive.clone()));
                    }
                }
            });
        });
}
