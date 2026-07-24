use crate::actions::UiAction;
use crate::app::App;

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let frame =
        egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(12, 8));
    egui::Panel::bottom("statusbar").frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            let selected = app.pane().selection.len();
            let summary = if selected > 0 {
                format!("{selected} élément(s) sélectionné(s)")
            } else {
                format!("{} élément(s)", app.pane().displayed_count())
            };
            ui.label(egui::RichText::new(summary).small().weak());
            if let Some(status) = &app.status {
                ui.separator();
                ui.label(egui::RichText::new(status.clone()).small());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let at_home = app.pane().is_home();
                if !at_home {
                    for (idx, external) in app.external_apps.iter().enumerate().rev() {
                        if ui
                            .button(&external.name)
                            .on_hover_text(format!("Ouvrir ici : {}", external.command))
                            .clicked()
                        {
                            actions.push(UiAction::OpenExternalApp(idx));
                        }
                    }
                    if ui
                        .button("🖳 Terminal")
                        .on_hover_text("Ouvrir un terminal ici")
                        .clicked()
                    {
                        actions.push(UiAction::OpenTerminal);
                    }
                    if ui
                        .button("</> Code")
                        .on_hover_text("Ouvrir dans VS Code")
                        .clicked()
                    {
                        actions.push(UiAction::OpenVsCode);
                    }
                }
                if !app.clipboard.is_empty() || selected > 0 {
                    ui.separator();
                }
                if !app.clipboard.is_empty() && ui.button("Coller").clicked() {
                    actions.push(UiAction::Paste);
                }
                if selected > 0 {
                    if ui.button("Couper").clicked() {
                        actions.push(UiAction::CopySelection { cut: true });
                    }
                    if ui.button("Copier").clicked() {
                        actions.push(UiAction::CopySelection { cut: false });
                    }
                }
            });
        });
    });
}
